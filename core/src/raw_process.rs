use ::std::io;
use ::nix::unistd::{Uid, Gid};
use ::sys_return::*;
use ::std::*;
use ::std::io::Write;
use ::std::fs::File;
use ::libc;
use ::std::ffi::CString;

pub use ::libc::{c_int, pid_t};
pub use ::libc::SIGCHLD;
pub use ::libc::{CLONE_NEWUSER, CLONE_NEWUTS, CLONE_NEWIPC, CLONE_NEWPID, CLONE_NEWNS,
                 CLONE_NEWNET, CLONE_PARENT, CLONE_NEWCGROUP};

pub struct RawProcess {
    pid: pid_t,
}

impl RawProcess {
    pub fn from_pid(pid: pid_t) -> RawProcess {
        RawProcess { pid }
    }

    pub unsafe fn raw_clone(flags: c_int) -> io::Result<Option<RawProcess>> {
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

    pub fn signal(&mut self, signum: c_int) -> io::Result<()> {
        unsafe { sys_return_unit(libc::kill(self.get_pid(), signum)) }
    }

    // Not currently used. Complex uid mappings should be written by a process with capabilities
    pub fn uid_map(&mut self) -> UidMapFactory {
        UidMapFactory::new(self)
    }

    // Not currently used. Gid mappings should be written by a process with capabilities
    pub fn gid_map(&mut self) -> GidMapFactory {
        GidMapFactory::new(self)
    }

    pub fn get_pid(&self) -> pid_t { self.pid }

    pub fn ptrace(&self) -> io::Result<()> {
        unsafe {
            sys_return_unit(libc::ptrace(libc::PTRACE_SEIZE, self.pid, 0/*ignored*/, 0/*ignored*/))
        }
    }

    pub fn ns_enter<S: AsRef<str>>(&self, ns_name: S) -> io::Result<()> {
        let path = format!("/proc/{}/ns/{}", self.pid, ns_name.as_ref());
        let path_c = CString::new(path.as_str()).unwrap();
        let ns_fd = sys_return_same(unsafe {
            ::libc::open(path_c.as_ptr(), ::libc::O_RDONLY | ::libc::O_CLOEXEC)
        })?;
        sys_return_unit(unsafe { ::libc::setns(ns_fd, 0) })?;
        sys_return_unit(unsafe { ::libc::close(ns_fd) })
    }

    // Consumes the process object
    pub fn ns_enter_mnt(self) -> io::Result<()> {
        self.ns_enter("mnt")
    }

    pub fn wait(self) -> io::Result<c_int> {
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

    pub fn set(self) -> io::Result<()> {
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

    pub fn set(self) -> io::Result<()> {
        File::create(&format!("/proc/{}/setgroups", self.process.get_pid()))?.write_all(b"deny")?;
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

    pub fn set(self, path: &str) -> io::Result<()> {
        if self.entries.is_empty() {
            return Ok(());
        }
        let mut buf = Vec::<u8>::new();
        for entry in self.entries {
            writeln!(&mut buf, "{} {} {}", entry.to, entry.from, entry.length)
                .ok().unwrap();
        }

        File::create(path)?.write_all(buf.as_ref())
    }
}
