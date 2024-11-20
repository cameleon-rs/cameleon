/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{
    borrow::Borrow,
    io::Cursor,
    net::{Ipv4Addr, UdpSocket},
    sync::{Arc, Condvar, Mutex},
    thread,
};

use cameleon_device::gige::protocol::stream::{
    ImageLeader, ImageTrailer, PacketHeader, PacketType, PayloadType, PayloadTypeKind,
};
use futures_channel::oneshot;
use tracing::{error, warn};

use crate::{
    payload::{Payload, PayloadSender},
    DeviceControl, PayloadStream, StreamError, StreamResult,
};

#[derive(Debug)]
pub struct StreamParams {
    pub host_addr: Ipv4Addr,
    pub host_port: u16,
}

pub struct StreamHandle {
    completion: Option<Arc<(Mutex<bool>, Condvar)>>,
    cancellation_tx: Option<oneshot::Sender<()>>,
    sock: Arc<UdpSocket>,
}

impl StreamHandle {
    pub fn new(sock: UdpSocket) -> StreamResult<Self> {
        Ok(Self {
            completion: None,
            cancellation_tx: None,
            sock: Arc::new(sock),
        })
    }

    pub fn port(&self) -> u16 {
        self.sock.local_addr().unwrap().port()
    }
}

impl PayloadStream for StreamHandle {
    fn open(&mut self) -> StreamResult<()> {
        // TODO:
        Ok(())
    }

    fn close(&mut self) -> StreamResult<()> {
        // TODO:
        Ok(())
    }

    fn start_streaming_loop(
        &mut self,
        sender: PayloadSender,
        _ctrl: &mut dyn DeviceControl,
    ) -> StreamResult<()> {
        if self.is_loop_running() {
            return Err(StreamError::InStreaming);
        }
        let (cancellation_tx, cancellation_rx) = oneshot::channel();
        self.cancellation_tx = Some(cancellation_tx);
        let completion = Arc::new((Mutex::new(false), Condvar::new()));
        self.completion = Some(completion.clone());

        // let stream_port = StreamRegister::new(0).channel_port(ctrl).unwrap().packet_size() as usize; // TODO: redo unwrap
        // let packet_size = StreamRegister::new(0).packet_size(ctrl).unwrap().host_port();
        let packet_size = 8996;

        let strm_loop = StreamingLoop {
            buffer: vec![0u8; packet_size],
            cancellation_rx,
            completion,
            sock: self.sock.clone(),
            sender,
        };

        thread::spawn(move || {
            strm_loop.run();
        });
        Ok(())
    }

    fn stop_streaming_loop(&mut self) -> StreamResult<()> {
        if let Some(cancel) = self.cancellation_tx.take() {
            if let Err(_) = cancel.send(()) {
                return Err(StreamError::Disconnected);
            }
            match self.completion.take().as_ref().map(Borrow::borrow) {
                Some((completion, condvar)) => {
                    let guard = completion.lock().unwrap();
                    if *guard {
                        return Ok(());
                    }
                    drop(condvar.wait_while(guard, |g| !*g).unwrap());
                    Ok(())
                }
                None => Err(StreamError::Disconnected),
            }
        } else {
            Ok(())
        }
    }

    fn is_loop_running(&self) -> bool {
        if let Some((completed, _)) = self.completion.as_ref().map(|v| v.as_ref()) {
            if !*completed.lock().unwrap() {
                return true;
            }
        }
        false
    }
}

struct StreamingLoop {
    buffer: Vec<u8>,
    cancellation_rx: oneshot::Receiver<()>,
    completion: Arc<(Mutex<bool>, Condvar)>,
    sock: Arc<UdpSocket>,
    sender: PayloadSender,
}

impl StreamingLoop {
    fn run(mut self) {
        macro_rules! unwrap_or_continue {
            ($expr:expr) => {
                match $expr {
                    Err(err) => {
                        use tracing::error;
                        error!(?err);
                        continue;
                    }
                    Ok(v) => v,
                }
            };
        }
        macro_rules! ensure_or_continue {
            ($expr:expr, $($tt:tt)*) => {
                if !($expr) {
                    use tracing::error;
                    error!($($tt)*);
                    continue;
                }
            };
        }
        let mut payload = Vec::new();
        let mut builder = None;

        macro_rules! handle_packet_mismatch {
            ($expr:expr) => {
                match $expr {
                    Ok(p) => p,
                    // if we get an old packet,
                    // we just ignore it and keep building
                    // the new frame
                    Err(PacketMismatch::TooOld) => continue,
                    Err(PacketMismatch::TooNew) => {
                        tracing::warn!("Packet loss occured, frame skipped");
                        builder = None;
                        continue;
                    }
                }
            };
        }

        loop {
            if self.cancellation_rx.try_recv().is_ok() {
                *self.completion.0.lock().unwrap() = true;
                self.completion.1.notify_all();
            }
            let length = self.sock.recv(&mut self.buffer).unwrap();
            let mut cursor = Cursor::new(&self.buffer[..]);
            let header = unwrap_or_continue!(PacketHeader::parse(&mut cursor));
            match header.packet_type {
                PacketType::Leader => {
                    let payload_type =
                        unwrap_or_continue!(PayloadType::parse_generic_leader(&mut cursor));
                    ensure_or_continue!(
                        payload_type.kind() == PayloadTypeKind::Image,
                        "Payload type kind: {:?} not suported",
                        payload_type.kind()
                    );
                    let leader = unwrap_or_continue!(ImageLeader::parse(&mut cursor));
                    if builder.is_some() {
                        warn!("A new leader packet has arrived while no trailer packet arrived");
                    }
                    builder = Some(PayloadBuilder::new(header, leader, &mut payload));
                }
                PacketType::Trailer => {
                    let payload_type =
                        unwrap_or_continue!(PayloadType::parse_generic_leader(&mut cursor));
                    let Some(builder) = builder.take() else {
                        warn!("Trailer packet received while no leader packet arrived");
                        continue;
                    };
                    ensure_or_continue!(
                        payload_type.kind() == PayloadTypeKind::Image,
                        "Payload type kind: {:?} not suported",
                        payload_type.kind()
                    );
                    let _trailer = unwrap_or_continue!(ImageTrailer::parse(&mut cursor));
                    let payload = handle_packet_mismatch!(builder.build(header));
                    unwrap_or_continue!(async_std::task::block_on(self.sender.send(Ok(payload))));
                }
                PacketType::GenericPayload => {
                    let Some(builder) = builder.as_mut() else {
                        warn!("Generic packet received while no leader packet arrived");
                        continue;
                    };
                    // TODO: migrate to Cursor::remaining_slice when it's stable
                    let remaining_slice = &self.buffer[cursor.position() as usize..length];
                    handle_packet_mismatch!(builder.push(header, remaining_slice));
                }
                PacketType::H264Payload => error!("H264 Payload not implemented"),
                PacketType::MultiZonePayload => error!("Multi Zone Payload not implemented"),
            };
        }
    }
}

struct PayloadBuilder<'a> {
    payload: &'a mut Vec<u8>,
    leader: ImageLeader,
    pub block_id: u64,
    last_packet_id: u32,
}

impl<'a> PayloadBuilder<'a> {
    fn new(header: PacketHeader, leader: ImageLeader, payload: &'a mut Vec<u8>) -> Self {
        payload.clear();
        Self {
            payload,
            leader,
            block_id: header.block_id,
            last_packet_id: 0,
        }
    }

    fn verify_header(&self, header: &PacketHeader) -> Result<(), PacketMismatch> {
        if header.block_id > self.block_id || header.packet_id > self.last_packet_id + 1 {
            // New frame or new packet, we can't wait, we quit
            return Err(PacketMismatch::TooNew);
        }
        if header.block_id < self.block_id {
            // We ignore packets from earlier presumed lost frames
            return Err(PacketMismatch::TooOld);
        }
        Ok(())
    }

    fn push(&mut self, header: PacketHeader, data: &[u8]) -> Result<(), PacketMismatch> {
        self.verify_header(&header)?;
        assert_eq!(header.packet_id, self.last_packet_id + 1);
        self.last_packet_id += 1;
        self.payload.extend_from_slice(data);
        Ok(())
    }

    fn build(self, header: PacketHeader) -> Result<Payload, PacketMismatch> {
        self.verify_header(&header)?;
        Ok(Payload {
            id: self.block_id,
            payload_type: crate::payload::PayloadType::Image,
            image_info: Some(crate::payload::ImageInfo {
                width: self.leader.width() as usize,
                height: self.leader.height() as usize,
                x_offset: self.leader.x_offset() as usize,
                y_offset: self.leader.y_offset() as usize,
                pixel_format: self.leader.pixel_format(),
                image_size: self.payload.len(),
            }),
            payload: self.payload.clone(),
            valid_payload_size: self.payload.len(),
            timestamp: self.leader.timestamp(),
        })
    }
}

enum PacketMismatch {
    TooNew,
    TooOld,
}
