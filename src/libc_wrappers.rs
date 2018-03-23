use ::std::*;
use ::std::io::Result;
use ::libc;
use ::sys_return::*;
use ::std::ffi::CString;

pub use ::libc::pid_t;
pub use ::libc::c_int;
pub use ::libc::c_void;
pub use ::libc::ssize_t;
pub use ::libc::{SIGCHLD, CLONE_CHILD_SETTID, CLONE_NEWUSER, CLONE_NEWUTS, CLONE_NEWIPC,
                 CLONE_NEWPID, CLONE_NEWNS, CLONE_NEWNET};

pub struct Process {
    pid: pid_t,
}

impl Process {
    pub unsafe fn raw_clone(flags: c_int) -> Result<Option<Process>> {
        let res = libc::syscall(libc::SYS_clone, flags,
                                /*stack-ptr*/ 0 as *mut (),
                                /*ptid*/ 0 as *mut (),
                                /*ctid*/ 0 as *mut (),
                                /*regs*/ 0 as *mut ());
        if res == 0 {
            return Ok(None)
        }
        Ok(Some(Process { pid: sys_return_same(res)? as pid_t }))
    }

    pub fn get_pid(&self) -> pid_t { self.pid }

    pub fn wait(self) -> Result<c_int> {
        unsafe {
            let mut status: c_int = 0;
            sys_return_unit(libc::waitpid(self.pid, &mut status, 0))?;
            Ok(libc::WEXITSTATUS(status))
        }
    }
}

pub unsafe fn sys_write(fd: c_int, data: &[u8]) -> Result<usize> {
    sys_return(libc::write(fd, data.as_ptr() as *const c_void, data.len()))
}

pub unsafe fn sys_read(fd: c_int, data: &mut [u8]) -> Result<usize> {
    sys_return(libc::read(fd, data.as_ptr() as *mut c_void, data.len()))
}

pub unsafe fn sys_close(fd: c_int) -> Result<()> {
    sys_return_unit(libc::close(fd))
}

pub fn set_hostname(hostname: &str) -> Result<()> {
    unsafe {
        let c_str = CString::new(hostname).unwrap();
        sys_return_unit(libc::sethostname(c_str.as_ptr(), c_str.as_bytes().len()))
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
