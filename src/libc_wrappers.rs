use ::std::*;
use ::std::io::Result;
use ::sys_return::*;
use ::std::ffi::CString;
use ::std::ptr::null;
use ::libc;

pub use ::libc::{pid_t, c_int, c_void, ssize_t, size_t, c_ulong, uid_t};
pub use ::libc::{MS_BIND, MS_REC};

pub unsafe fn sys_write(fd: c_int, data: &[u8]) -> Result<usize> {
    sys_return(libc::write(fd, data.as_ptr() as *const c_void, data.len()))
}

pub unsafe fn sys_read(fd: c_int, data: &mut [u8]) -> Result<usize> {
    sys_return(libc::read(fd, data.as_ptr() as *mut c_void, data.len()))
}

pub unsafe fn sys_close(fd: c_int) -> Result<()> {
    sys_return_unit(libc::close(fd))
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

pub struct Pipe {
    read_fd: c_int,
    write_fd: c_int,
}

impl Drop for Pipe {
    fn drop(&mut self) {
        unsafe {
            sys_close(self.read_fd).expect("ERROR closing pipe");
            sys_close(self.write_fd).expect("ERROR closing pipe");
        }
    }
}

impl io::Write for Pipe {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        unsafe { sys_write(self.write_fd, buf) }
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl io::Read for Pipe {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        unsafe { sys_read(self.read_fd, buf) }
    }
}

impl Pipe {
    pub fn new() -> Result<Pipe> {
        let mut pipe_fd: [c_int; 2] = [0, 0];
        let res = unsafe { libc::pipe(pipe_fd.as_mut_ptr()) };
        sys_return_unit(res)?;
        Ok(Pipe { read_fd: pipe_fd[0], write_fd: pipe_fd[1] })
    }
}
