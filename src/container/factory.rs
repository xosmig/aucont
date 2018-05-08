use ::*;
use ::raw_process::*;
use ::std::*;
use ::std::io::Write;
use ::nix::unistd::{getuid, getgid, Uid, Gid};
use ::std::ffi::OsStr;
use ::std::net::Ipv4Addr;
use super::{Error, Result, Container, CommentError};
use super::container_init_main::*;

pub struct NetworkConfig {
    pub cont_addr: Ipv4Addr,
    pub host_addr: Ipv4Addr,
}

pub struct ContainerConfig {
    pub daemonize: bool,
    pub image_path: String,
    pub cmd: String,
    pub cmd_args: Vec<String>,
    pub net: Option<NetworkConfig>,
}

pub struct ContainerFactory {
    config: ContainerConfig,
    process: RawProcess,
    pipe: Pipe,
}

fn shell<I, S1, S2>(cmd: S1, args: I) -> Result<()>
    where I: IntoIterator<Item=S2> + std::fmt::Debug, S1: AsRef<str>, S2: AsRef<OsStr>
{
    let error_string = format!("Error executing '{}' with arguments {:?}", cmd.as_ref(), args);

    let output = process::Command::new(cmd.as_ref())
        .args(args)
        .output().comment_error(error_string.as_str())?;

    if !output.status.success() {
        let stderr_string = std::str::from_utf8(&output.stderr).unwrap();
        let stdout_string = std::str::from_utf8(&output.stdout).unwrap();
        let error_message = format!("{}\nstderr:\n{}\nstdout:\n{}\n=======",
                                    error_string.as_str(), stderr_string, stdout_string);
        return Err(Error::simple(error_message));
    }

    Ok(())
}

macro_rules! sudo {
    ( $( $x: expr ),* ) => {
        shell("sudo", &[ $( $x ),* ])
    };
}

impl ContainerFactory {
    pub fn new_container(config: ContainerConfig) -> Result<Container> {
        let mut factory = ContainerFactory::new(config)?;
        factory.map_uid()?;
        factory.init_dir()?;
        factory.copy_rootfs()?;
        factory.record_info()?;
        factory.configure_network()?;
        factory.start_init()?;
        factory.finish()
    }

    pub fn new(config: ContainerConfig) -> Result<Self> {
        let pipe = Pipe::new().comment_error("ERROR creating pipe")?;

        let process = unsafe {
            RawProcess::raw_clone(SIGCHLD | CLONE_NEWNS | CLONE_NEWUSER | CLONE_NEWUTS |
                CLONE_NEWIPC | CLONE_NEWPID | CLONE_NEWNET | CLONE_NEWCGROUP)
        }.comment_error("Error creating init process for the container")?;

        if process.is_none() {
            container_init_main(
                pipe,
                ContainerInitConfig {
                    cmd: config.cmd.clone(),
                    cmd_args: config.cmd_args.clone(),
                    daemonize: config.daemonize,
                },
            );
            // unreachable
        }

        Ok(ContainerFactory {
            process: process.unwrap(),
            pipe,
            config,
        })
    }

    pub fn get_id(&self) -> pid_t {
        self.process.get_pid()
    }

    pub fn map_uid(&mut self) -> Result<()> {
        self.process.uid_map()
            .entry(getuid(), Uid::from_raw(0))
            .set().comment_error("Internal error: cannot set UID mapping")?;

        self.process.gid_map()
            .entry(getgid(), Gid::from_raw(0))
            .set().comment_error("Internal error: cannot set GID mapping")?;

        Ok(())
    }

    pub fn init_dir(&mut self) -> Result<()> {
        let dir: &str = &container_dir(self.get_id());

        if path::Path::new(dir).exists() {
            return Err(Error::simple("Internal error ('{}' already exists)"));
        }

        fs::create_dir_all(dir).comment_error("Internal error (create container dir)")?;
        Ok(())
    }

    pub fn copy_rootfs(&mut self) -> Result<()> {
        let root_fs = container_root_fs(self.get_id());
        sudo!("cp", "--recursive", "--one-file-system", "--preserve",
            &self.config.image_path, root_fs.as_str())?;
        Ok(())
    }

    pub fn record_info(&mut self) -> Result<()> {
        let info_dir = &container_info_dir(self.get_id());
        fs::create_dir_all(info_dir).comment_error("Internal error (create info dir)")?;

        let mut daemon_file = fs::File::create(&format!("{}/daemon", info_dir))
            .comment_error("Internal error (open daemon file)")?;
        writeln!(daemon_file, "{}", if self.config.daemonize { 1 } else { 0 })
            .comment_error("Internal error (write daemon file)")?;

        Ok(())
    }

    pub fn start_init(&mut self) -> Result<()> {
        // send the pid to the init process
        write!(self.pipe, "{}", self.process.get_pid())
            .comment_error("Internal error (writing PID to pipe)")?;
        Ok(())
    }

    pub fn configure_network(&mut self) -> Result<()> {
        if let Some(ref conf) = self.config.net {
            let id = &self.get_id().to_string() as &str;
            let guest_ip = &conf.cont_addr.to_string() as &str;
            let host_ip = &conf.host_addr.to_string() as &str;
            let veth_host = &format!("veth{}h", id) as &str;
            let veth_guest = &format!("veth{}g", id) as &str;

            sudo!("ip", "link", "add", veth_host, "type", "veth", "peer", "name", veth_guest)?;
            sudo!("ip", "link", "set", veth_guest, "netns", id)?;
            sudo!("nsenter", "--net", "-t", id, "ip", "link", "set", "lo", "up")?;
            sudo!("nsenter", "--net", "-t", id, "ip", "link", "set", veth_guest, "name", "eth0")?;
            sudo!("nsenter", "--net", "-t", id, "ip", "link", "set", "eth0", "up")?;
            sudo!("nsenter", "--net", "-t", id, "ip", "addr", "add",
                &format!("{}/24", guest_ip), "dev", "eth0")?;
            sudo!("ip", "link", "set", veth_host, "up")?;
            sudo!("ip", "addr", "add", host_ip, "dev", veth_host)?;
            sudo!("ip", "route", "add", guest_ip, "dev", veth_host)?;
        }
        Ok(())
    }

    pub fn finish(self) -> Result<Container> {
        // TODO: wait for init process to finish initialization
        Ok(Container {
            process: self.process,
            is_daemon: self.config.daemonize,
        })
    }
}
