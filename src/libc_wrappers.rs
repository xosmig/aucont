use ::std::os::unix::io::RawFd;
use ::std::io;
use ::sys_return::*;
use ::std::ffi::CString;
use ::std::ptr::null;
use ::libc;

pub use ::libc::{pid_t, c_int, c_void, ssize_t, size_t, c_ulong, uid_t, gid_t};
pub use ::libc::{MS_BIND, MS_REC};
pub use ::libc_ext::*;

pub fn sys_write(fd: RawFd, data: &[u8]) -> io::Result<usize> {
    unsafe { sys_return(libc::write(fd, data.as_ptr() as *const c_void, data.len())) }
}

pub fn sys_read(fd: RawFd, data: &mut [u8]) -> io::Result<usize> {
    unsafe { sys_return(libc::read(fd, data.as_ptr() as *mut c_void, data.len())) }
}

pub fn sys_close(fd: RawFd) -> io::Result<()> {
    unsafe { sys_return_unit(libc::close(fd)) }
}

pub fn sys_mount(src: &str, target: &str, fstype: &str, flags: c_ulong, options: Option<&str>)
                 -> io::Result<()> {
    unsafe {
        let src_c = CString::new(src).unwrap();
        let target_c = CString::new(target).unwrap();
        let fstype_c = CString::new(fstype).unwrap();
        let options_c = options.map(|opt_str| CString::new(opt_str).unwrap());
        let options_ptr = options_c.as_ref().map(|c_str| c_str.as_ptr()).unwrap_or(null());
        sys_return_unit(libc::mount(
            src_c.as_ptr(), target_c.as_ptr(), fstype_c.as_ptr(), flags,
            options_ptr as *const c_void))
    }
}

pub fn sys_chown(path: &str, uid: uid_t, gid: gid_t) -> io::Result<()> {
    unsafe {
        let path_c = CString::new(path).unwrap();
        sys_return_unit(::libc::chown(path_c.as_ptr(), uid, gid))
    }
}

pub fn sys_umount(path: &str) -> io::Result<()> {
    unsafe {
        let path_c = CString::new(path).unwrap();
        sys_return_unit(::libc::umount(path_c.as_ptr()))
    }
}

pub fn getpid() -> pid_t {
    unsafe { ::libc::getpid() }
}
