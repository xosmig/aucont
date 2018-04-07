use ::std::io::Result;
use ::nix::unistd::{Uid, Gid};
use ::sys_return::*;
use ::std::*;
use ::std::io::Write;
use ::std::fs::File;
use ::libc;

pub use ::libc::{c_int, pid_t};
pub use ::libc::SIGCHLD;
pub use ::libc::{CLONE_NEWUSER, CLONE_NEWUTS, CLONE_NEWIPC, CLONE_NEWPID, CLONE_NEWNS,
                 CLONE_NEWNET, CLONE_PARENT};

pub struct RawProcess {
    pid: pid_t,
}

impl RawProcess {
    pub fn from_pid(pid: pid_t) -> Result<RawProcess> {
        Ok(RawProcess { pid })
    }

    pub unsafe fn raw_clone(flags: c_int) -> Result<Option<RawProcess>> {
        let res = libc::syscall(libc::SYS_clone, flags,
                                /*stack-ptr*/ 0 as *mut (),
                                /*ptid*/ 0 as *mut (),
                                /*ctid*/ 0 as *mut (),
                                /*regs*/ 0 as *mut ());
        if res == 0 {
            return Ok(None);
        }
        Ok(Some(RawProcess { pid: sys_return_same(res)? as pid_t }))
    }

    pub fn signal(&mut self, signum: c_int) -> Result<()> {
        unsafe { sys_return_unit(libc::kill(self.get_pid(), signum)) }
    }

    pub fn uid_map(&mut self) -> UidMapFactory {
        UidMapFactory::new(self)
    }

    pub fn gid_map(&mut self) -> GidMapFactory {
        GidMapFactory::new(self)
    }

    pub fn get_pid(&self) -> pid_t { self.pid }

    pub fn ptrace(&self) -> Result<()> {
        unsafe {
            sys_return_unit(libc::ptrace(libc::PTRACE_SEIZE, self.pid, 0/*ignored*/, 0/*ignored*/))
        }
    }

    pub fn wait(self) -> Result<c_int> {
        unsafe {
            let mut status: c_int = 0;
            sys_return_unit(libc::waitpid(self.pid, &mut status, 0))?;
            Ok(libc::WEXITSTATUS(status))
        }
    }
}

#[must_use]
pub struct UidMapFactory<'a> {
    process: &'a mut RawProcess,
    factory: IdMapFactory<Uid>,
}

impl<'a> UidMapFactory<'a> {
    pub fn new(process: &'a mut RawProcess) -> Self {
        UidMapFactory { process, factory: IdMapFactory::new() }
    }

    pub fn entry(mut self, from: Uid, to: Uid) -> Self {
        self.factory.entry(from, to);
        self
    }

    pub fn set(self) -> Result<()> {
        self.factory.set(&format!("/proc/{}/uid_map", self.process.get_pid()))
    }
}

#[must_use]
pub struct GidMapFactory<'a> {
    process: &'a mut RawProcess,
    factory: IdMapFactory<Gid>,
}

impl<'a> GidMapFactory<'a> {
    pub fn new(process: &'a mut RawProcess) -> Self {
        GidMapFactory { process, factory: IdMapFactory::new() }
    }

    pub fn entry(mut self, from: Gid, to: Gid) -> Self {
        self.factory.entry(from, to);
        self
    }

    pub fn set(self) -> Result<()> {
        {
            let mut file = File::create(&format!("/proc/{}/setgroups", self.process.get_pid()))?;
            write!(file, "deny").expect("Internal error (disabling setgroups for container)");
        }
        self.factory.set(&format!("/proc/{}/gid_map", self.process.get_pid()))
    }
}

#[must_use]
pub struct IdMapFactory<I> {
    entries: Vec<IdMapEntry<I>>,
}

struct IdMapEntry<I> {
    from: I,
    to: I,
    length: u32,
}

impl<I: fmt::Display> IdMapFactory<I> {
    pub fn new() -> Self {
        IdMapFactory { entries: vec![] }
    }

    pub fn entry(&mut self, from: I, to: I) {
        self.entries.push(IdMapEntry { from, to, length: 1 });
    }

    pub fn set(self, path: &str) -> Result<()> {
        if self.entries.is_empty() {
            return Ok(());
        }
        let mut buf = Vec::<u8>::new();
        for entry in self.entries {
            writeln!(&mut buf, "{} {} {}", entry.to, entry.from, entry.length)
                .ok().unwrap();
        }

        let mut file = File::create(path)?;
        file.write_all(buf.as_ref())
    }
}
