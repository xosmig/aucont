pub mod factory;
pub mod result;
mod container_init_main;

pub use ::raw_process::pid_t;

use ::std::fs;
use ::raw_process::*;
use ::{container_dir, container_info_dir, read_number};
use self::result::{Error, Result, CommentError};


pub struct Container {
    process: RawProcess,
    is_daemon: bool,
}

impl Container {
    pub fn connect(id: pid_t) -> Result<Container> {
        let mut daemon_file = fs::File::open(container_info_dir(id) + "/daemon")
            .comment("Cannot read container info (is_daemon)")?;
        let daemon_number = read_number(&mut daemon_file).comment("Parsing daemon file")?;

        let res = Container {
            process: RawProcess::from_pid(id)?,
            is_daemon: daemon_number != 0,
        };

        res.process.ptrace().comment("Ptrace")?;
        Ok(res)
    }

    pub fn cancel(&mut self, signal: c_int) -> Result<()> {
        Ok(self.process.signal(signal)?)
    }

    pub fn wait_and_clear(self) -> Result<c_int> {
        let id = self.get_id();
        let ret = self.process.wait().comment("Waiting for process to finish")?;
        fs::remove_dir_all(&container_dir(id)).comment("Removing container files")?;
        Ok(ret)
    }

    pub fn get_id(&self) -> pid_t {
        self.process.get_pid()
    }

    pub fn is_daemon(&self) -> bool {
        self.is_daemon
    }
}
