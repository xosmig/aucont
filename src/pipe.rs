use ::std::io::{self, Result};
use ::std::os::unix::io::RawFd;
use ::libc_wrappers::{sys_read, sys_write, sys_close};
use ::libc;
use ::sys_return::*;
use ::check::Check;

pub struct Pipe {
    read_fd: RawFd,
    write_fd: RawFd,
}

impl Drop for Pipe {
    fn drop(&mut self) {
        sys_close(self.read_fd).log_error("ERROR closing pipe");
        sys_close(self.write_fd).log_error("ERROR closing pipe");
    }
}

impl io::Write for Pipe {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        sys_write(self.write_fd, buf)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl io::Read for Pipe {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        sys_read(self.read_fd, buf)
    }
}

impl Pipe {
    pub fn new() -> Result<Pipe> {
        let mut pipe_fd: [RawFd; 2] = [0, 0];
        let res = unsafe { libc::pipe2(pipe_fd.as_mut_ptr(), libc::O_CLOEXEC) };
        sys_return_unit(res)?;
        Ok(Pipe { read_fd: pipe_fd[0], write_fd: pipe_fd[1] })
    }
}
