use ::sys_return::*;
use ::std::io;
use ::std::ffi::CString;

mod raw {
    use ::libc::{c_char, c_int};

    extern "C" {
        pub fn redirect_stdin(path: *const c_char) -> c_int;
        pub fn redirect_stdout(path: *const c_char) -> c_int;
        pub fn redirect_stderr(path: *const c_char) -> c_int;
        pub fn redirect_stderr_to_stdout() -> c_int;
    }
}

pub fn redirect_stdin<S: AsRef<str>>(path: S) -> io::Result<()> {
    unsafe {
        let path_c = CString::new(path.as_ref()).unwrap();
        sys_return_unit(raw::redirect_stdin(path_c.as_ptr()))
    }
}

pub fn redirect_stdout<S: AsRef<str>>(path: S) -> io::Result<()> {
    unsafe {
        let path_c = CString::new(path.as_ref()).unwrap();
        sys_return_unit(raw::redirect_stdout(path_c.as_ptr()))
    }
}

pub fn redirect_stderr<S: AsRef<str>>(path: S) -> io::Result<()> {
    unsafe {
        let path_c = CString::new(path.as_ref()).unwrap();
        sys_return_unit(raw::redirect_stderr(path_c.as_ptr()))
    }
}

pub fn redirect_stderr_to_stdout() -> io::Result<()> {
    unsafe { sys_return_unit(raw::redirect_stderr_to_stdout()) }
}
