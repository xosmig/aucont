use ::std::*;
use ::process::*;
use ::{container_dir, container_info_dir, read_number};

pub use ::process::pid_t;

pub struct Container {
    process: Process,
}

impl Container {
    pub unsafe fn new(id: pid_t) -> io::Result<Container> {
        Ok(Container { process: Process::from_pid(id)? })
    }

    pub fn wait(self) -> io::Result<c_int> {
        let id = self.get_id();
        let ret = self.process.wait()?;
        fs::remove_dir_all(&container_dir(id))?;
        Ok(ret)
    }

    pub fn get_id(&self) -> pid_t {
        self.process.get_pid()
    }

    pub fn is_daemon(&self) -> bool {
        let mut file = fs::File::open(container_info_dir(self.get_id()) + "/daemon")
            .expect("Cannot read container info (is_daemon)");
        if read_number(&mut file).expect("Internal error (is daemon)") != 0 { true } else { false }
    }
}
