mod container_init_main;
pub mod factory;
pub mod result;

pub use ::raw_process::pid_t;

use ::std::fs;
use ::raw_process::*;
use ::{container_dir, container_info_dir, read_number};
use self::result::{Error, Result, CommentError};


pub struct Container {
    process: RawProcess,
    is_daemon: bool,
}

fn suppress_esrch(res: ::std::io::Result<()>) -> ::std::io::Result<()> {
    match res {
        Err(e) => {
            match e.raw_os_error() {
                Some(::libc::ESRCH) => Ok(()),
                _ => Err(e),
            }
        },
        Ok(()) => Ok(()),
    }
}

impl Container {
    pub fn connect(id: pid_t) -> Result<Container> {
        let mut daemon_file = fs::File::open(container_info_dir(id) + "/daemon")
            .comment_error("Cannot read container info (is_daemon)")?;
        let daemon_number = read_number(&mut daemon_file)
            .comment_error("Parsing daemon file")?;

        let res = Container {
            process: RawProcess::from_pid(id),
            is_daemon: daemon_number != 0,
        };
        suppress_esrch(res.process.ptrace()).comment_error("Ptrace")?;

        Ok(res)
    }

    pub fn cancel(&mut self, signal: c_int) -> Result<()> {
        suppress_esrch(self.process.signal(signal)).comment_error("Error killing process")?;
        Ok(())
    }

    pub fn wait_and_clear(self) -> Result<c_int> {
        let id = self.get_id();
        let ret = match self.process.wait() {
            Err(e) => match e.raw_os_error() {
                Some(::libc::ECHILD) => { 0 },
                _ => return Err(e).comment_error("Waiting for process to finish"),
            },
            Ok(code) => code,
        };
        ::cgroup::cgroup_delete(id).comment_error("Error removing cgroup")?;
        fs::remove_dir_all(&container_dir(id)).comment_error("Removing container files")?;
        Ok(ret)
    }

    pub fn get_id(&self) -> pid_t {
        self.process.get_pid()
    }

    pub fn is_daemon(&self) -> bool {
        self.is_daemon
    }
}
