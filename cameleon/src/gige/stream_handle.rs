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
    ImageLeader, ImageTrailer, PacketHeader, PacketType,
};
use futures_channel::oneshot;
use tracing::{error, warn};

use crate::{
    payload::{Payload, PayloadSender},
    DeviceControl, PayloadStream, StreamError, StreamResult,
};

use super::register_map::{Bootstrap, StreamRegister};

pub struct StreamHandle {
    completion: Option<Arc<(Mutex<bool>, Condvar)>>,
    cancellation_tx: Option<oneshot::Sender<()>>,
}

impl StreamHandle {
    pub fn new() -> StreamResult<Self> {
        Ok(Self {
            completion: None,
            cancellation_tx: None,
        })
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
        _sender: PayloadSender,
        ctrl: &mut dyn DeviceControl,
    ) -> StreamResult<()> {
        if self.is_loop_running() {
            return Err(StreamError::InStreaming);
        }
        let (cancellation_tx, cancellation_rx) = oneshot::channel();
        self.cancellation_tx = Some(cancellation_tx);
        let completion = Arc::new((Mutex::new(false), Condvar::new()));
        self.completion = Some(completion.clone());

        let stream_port = StreamRegister::new(0).channel_port(ctrl).unwrap(); // TODO: redo unwrap
        let packet_size = StreamRegister::new(0).packet_size(ctrl).unwrap();

        let strm_loop = StreamingLoop {
            buffer: vec![0u8; packet_size.packet_size() as usize],
            cancellation_rx,
            completion,
            sock: UdpSocket::bind(("0.0.0.0", stream_port.host_port()))?,
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
                    condvar.wait_while(guard, |g| !*g).unwrap();
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
    sock: UdpSocket,
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
        let mut payload = Vec::new();
        let mut builder = None;
        loop {
            self.sock.recv(&mut self.buffer).unwrap();
            let mut cursor = Cursor::new(&self.buffer[..]);
            let header = unwrap_or_continue!(PacketHeader::parse(&mut cursor));
            match header.packet_type {
                PacketType::Leader => {
                    let leader = unwrap_or_continue!(ImageLeader::parse(&mut cursor));
                    if builder.is_some() {
                        warn!("A new leader packet has arrived while no trailer packet arrived");
                    }
                    builder = Some(PayloadBuilder::new(leader, &mut payload));
                }
                PacketType::Trailer => {
                    let Some(payload) = builder.take() else {
                        warn!("Trailer packed received while no leader packet arrived");
                        continue;
                    };
                    let trailer = unwrap_or_continue!(ImageTrailer::parse(&mut cursor));
                    let payload = payload.build(trailer);
                    unwrap_or_continue!(self.sender.send(Ok(payload)));
                }
                PacketType::GenericPayload => {
                    builder.push();
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
}

impl<'a> PayloadBuilder<'a> {
    pub fn new(leader: ImageLeader, payload: &'a mut Vec<u8>) -> Self {
        payload.clear();
        Self { payload, leader }
    }

    pub fn push(packet: ImageGeneric) {
        //
    }

    pub fn build(self, trailer: ImageTrailer) -> Payload {
        Payload {
            id: todo!(),
            payload_type: crate::payload::PayloadType::Image,
            image_info: Some(crate::payload::ImageInfo {
                width: (),
                height: (),
                x_offset: (),
                y_offset: (),
                pixel_format: (),
                image_size: (),
            }),
            payload: todo!(),
            valid_payload_size: todo!(),
            timestamp: todo!(),
        }
    }
}
