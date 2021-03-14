use std::{ffi::{CString, c_void}, path::Path};

use thiserror::Error;

use chdr_sys::{_chd_error_CHDERR_NONE, chd_close, chd_file, chd_get_header, chd_header, chd_open, chd_read};

#[derive(Debug, Error)]
pub enum ChdError {
    #[error("chd_error: `{0}`")]
    ChdrCError(chdr_sys::chd_error),
    #[error("File does not exist")]
    FileDoesntExistError,
    #[error("Path encoding error")]
    IncompatiblePathEncodingError,
    #[error("Provided buffer is too small")]
    BufferTooSmallError,
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
            return Err(ChdError::FileDoesntExistError);
        }
        // TODO: This is wrong (enforces UTF-8), but what would be a more correct way
        // of passing this to libchdr?
        let path_str = path.as_ref().to_str();
        if path_str.is_none() {
            return Err(ChdError::IncompatiblePathEncodingError);
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

        let chd_header = unsafe {
            *chd_get_header(chd)
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
            return Err(ChdError::BufferTooSmallError);
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
}

impl Drop for ChdFile {
    fn drop(&mut self) {
        unsafe {
            chd_close(self.chd);
        }
    }
}