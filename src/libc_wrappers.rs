use ::std::os::unix::io::RawFd;
use ::std::*;
use ::std::io::Result;
use ::sys_return::*;
use ::std::ffi::CString;
use ::std::ptr::null;
use ::libc;

pub use ::libc::{pid_t, c_int, c_void, ssize_t, size_t, c_ulong, uid_t};
pub use ::libc::{MS_BIND, MS_REC};

pub fn sys_write(fd: RawFd, data: &[u8]) -> Result<usize> {
    unsafe { sys_return(libc::write(fd, data.as_ptr() as *const c_void, data.len())) }
}

pub fn sys_read(fd: RawFd, data: &mut [u8]) -> Result<usize> {
    unsafe { sys_return(libc::read(fd, data.as_ptr() as *mut c_void, data.len())) }
}

pub fn sys_close(fd: RawFd) -> Result<()> {
    unsafe { sys_return_unit(libc::close(fd)) }
}

pub fn sys_mount(src: &str, target: &str, fstype: &str, flags: c_ulong) -> Result<()> {
    unsafe {
        let src_c = CString::new(src).unwrap();
        let target_c = CString::new(target).unwrap();
        let fstype_c = CString::new(fstype).unwrap();
        sys_return_unit(libc::mount(
            src_c.as_ptr(), target_c.as_ptr(), fstype_c.as_ptr(), flags, null()))
    }
}
