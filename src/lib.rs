pub mod metadata;

use std::{ffi::{CString, c_void}, path::Path};

use thiserror::Error;

use chdr_sys::{_chd_error_CHDERR_NONE, chd_file, chd_header};
use chdr_sys::{chd_close, chd_open};
use chdr_sys::{chd_get_header, chd_get_metadata, chd_read};

use metadata::{CDROM_TRACK_METADATA_TAG,CDROM_TRACK_METADATA2_TAG, CdTrackInfo};

#[derive(Debug, Error)]
pub enum ChdError {
    #[error("chd_error: `{0}`")]
    ChdrCError(chdr_sys::chd_error),
    #[error("File does not exist")]
    FileDoesntExist,
    #[error("Path encoding error")]
    IncompatiblePathEncoding,
    #[error("Header is null")]
    HeaderNull,
    #[error("Provided buffer is too small")]
    BufferTooSmall,
    #[error("Couldn't parse CHD metadata")]
    MetadataTextParseError(#[from] text_io::Error),
    #[error("No CDROM metadata present")]
    NoCdromMetadata,
}

pub struct ChdFile {
    chd: *mut chd_file,
    chd_header: chd_header,
}

impl ChdFile {
    pub fn open<P>(path: P) -> Result<ChdFile, ChdError>
        where P: AsRef<Path>
    {
        if !path.as_ref().exists() {
            return Err(ChdError::FileDoesntExist);
        }
        // TODO: This is wrong (enforces UTF-8), but what would be a more correct way
        // of passing this to libchdr?
        let path_str = path.as_ref().to_str();
        if path_str.is_none() {
            return Err(ChdError::IncompatiblePathEncoding);
        }
        let path_cstring = CString::new(path_str.unwrap()).unwrap();
        let mut chd: *mut chd_file = std::ptr::null_mut();
        let ret = unsafe {
            chd_open(path_cstring.as_ptr(),
                chdr_sys::CHD_OPEN_READ as i32,
                std::ptr::null_mut(),
                &mut chd
            )
        };

        if ret != _chd_error_CHDERR_NONE {
            return Err(ChdError::ChdrCError(ret));
        }

        let chd_header_ptr = unsafe {
            chd_get_header(chd)
        };

        let chd_header = if !chd_header_ptr.is_null() {
            unsafe {
                *chd_header_ptr
            }
        } else {
            return Err(ChdError::HeaderNull);
        };

        Ok(ChdFile {
            chd,
            chd_header,
        })
    }

    pub fn hunk_len(&self) -> u32 {
        // TODO: What is "unitbytes"?
        self.chd_header.hunkbytes
    }

    pub fn num_hunks(&self) -> u32 {
        // TODO: What is "hunkcount"?
        self.chd_header.totalhunks
    }

    #[must_use]
    pub fn read_hunk(&mut self, index: u32, buffer: &mut [u8]) -> Result<(), ChdError> {
        if buffer.len() < self.hunk_len() as usize {
            return Err(ChdError::BufferTooSmall);
        }
        let ret = unsafe {
            chd_read(self.chd, index, buffer.as_mut_ptr() as *mut c_void)
        };
        if ret != _chd_error_CHDERR_NONE {
            Err(ChdError::ChdrCError(ret))
        } else {
            Ok(())
        }
    }

    pub fn cd_tracks(&self) -> Result<Vec<CdTrackInfo>, ChdError> {
        let mut tracks = Vec::new();
        let mut metadata_buf = [0u8; 1024];
        let mut resultlen = 0u32;

        // Search for new metadata first
        let mut searchtag: u32 = CDROM_TRACK_METADATA2_TAG;
        loop {
            let ret = unsafe {
                chd_get_metadata(self.chd,
                    searchtag,
                    tracks.len() as u32,
                    metadata_buf.as_mut_ptr() as *mut c_void,
                    metadata_buf.len() as u32,
                    &mut resultlen,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                )
            };

            if ret == chdr_sys::_chd_error_CHDERR_METADATA_NOT_FOUND {
                if tracks.is_empty() && searchtag == CDROM_TRACK_METADATA2_TAG {
                    // We haven't found any tracks and are still looking for metadata version
                    // 2, so let's switch over to version 1
                    searchtag = CDROM_TRACK_METADATA_TAG;
                } else {
                    // Either we've found tracks or we've tried version 1 already
                    break;
                }
            } else if ret != _chd_error_CHDERR_NONE {
                return Err(ChdError::ChdrCError(ret));
            }

            let track = if searchtag == CDROM_TRACK_METADATA2_TAG {
                // - 1 to cut off null byte
                CdTrackInfo::from_metadata(&metadata_buf[..resultlen as usize - 1])?
            } else {
                // - 1 to cut off null byte
                CdTrackInfo::from_old_metadata(&metadata_buf[..resultlen as usize - 1])?
            };
            tracks.push(track);
        }

        if tracks.is_empty() {
            Err(ChdError::NoCdromMetadata)
        } else {
            Ok(tracks)
        }
    }
}

impl Drop for ChdFile {
    fn drop(&mut self) {
        unsafe {
            chd_close(self.chd);
        }
    }
}