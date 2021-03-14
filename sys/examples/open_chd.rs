use chdr_sys::chd_file;
use chdr_sys::CHD_OPEN_READ;
use chdr_sys::{chd_close, chd_open, chd_precache};

use std::ffi::CString;

fn main() {
    let mut args = std::env::args();
    if args.len() < 2 {
        panic!("No CHD filename supplied. Usage: {:?} <CHD file>", std::env::current_exe().unwrap());
    }

    let chd_filename = args.nth(1).unwrap();
    println!("CHD file: {}", chd_filename);
    let chd_filename_cstring = CString::new(chd_filename).unwrap();

    let mut chd: *mut chd_file = std::ptr::null_mut();

    let e = unsafe {
        chd_open(chd_filename_cstring.as_ptr(),
            CHD_OPEN_READ as i32,
            std::ptr::null_mut(),
            &mut chd
        )
    };

    if e != 0 {
        panic!("Error opening CHD file: {}", e);
    }
    
    println!("Successfully opened CHD file.");
    println!("Precaching...");
    unsafe {
        chd_precache(chd);
    }
    println!("Done!");

    unsafe {
        chd_close(chd);
    }
    println!("Closed CHD file.");
}