/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{convert::TryInto, io::Read};

use super::{ControlError, ControlResult};

macro_rules! unwrap_or_log {
    ($expr:expr) => {{
        use tracing::error;
        match $expr {
            Ok(v) => v,
            Err(error) => {
                error!(?error);
                return Err(error.into());
            }
        }
    }};
}

pub(crate) fn unzip_genxml(file: Vec<u8>) -> ControlResult<Vec<u8>> {
    fn zip_err(err: impl std::fmt::Debug) -> ControlError {
        ControlError::InvalidDevice(format!("zipped xml file is broken: {:?}", err).into())
    }

    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(file)).unwrap();
    if zip.len() != 1 {
        return Err(zip_err("more than one files in zipped GenApi XML"));
    }
    let mut file = unwrap_or_log!(zip.by_index(0).map_err(zip_err));
    let file_size: usize = unwrap_or_log!(file.size().try_into());
    let mut xml = Vec::with_capacity(file_size);
    unwrap_or_log!(file.read_to_end(&mut xml).map_err(zip_err));
    Ok(xml)
}
