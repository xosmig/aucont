use ::core::Pipe;
use ::core::aucont_paths::*;
use ::core::raw_process::*;
use ::core::libc_wrappers::{getuid, getgid, Uid, Gid};
use ::std::*;
use ::std::io::Write;
use ::std::net::Ipv4Addr;
use super::{Error, Result, Container, CommentError};
use super::container_init_main::*;
use ::cgroup::cgroup_create;

pub struct NetworkConfig {
    pub cont_addr: Ipv4Addr,
    pub host_addr: Option<Ipv4Addr>,
    pub host_bridge: Option<String>,
}

#[derive(Default)]
pub struct ContainerConfig {
    pub daemonize: bool,
    pub image_path: String,
    pub cmd: String,
    pub cmd_args: Vec<String>,
    pub net: Option<NetworkConfig>,
    pub cpu_perc: Option<u32>,

    pub environment: Vec<(String, String)>,
    pub redirect_stderr: Option<String>,
    pub redirect_stdin: Option<String>,
    pub redirect_stdout: Option<String>,
}

pub struct ContainerFactory {
    config: ContainerConfig,
    process: RawProcess,
    pipe: Pipe,
}


impl ContainerFactory {
    pub fn new_container(config: ContainerConfig) -> Result<Container> {
        let mut factory = ContainerFactory::new(config)?;
        factory.map_uid()?;
        factory.init_dir()?;
        factory.copy_rootfs()?;
        factory.record_info()?;
        factory.set_cpu_limit()?;
        factory.configure_network()?;
        factory.start_init()?;
        factory.finish()
    }

    pub fn new(config: ContainerConfig) -> Result<Self> {
        let pipe = Pipe::new().comment_error("ERROR creating pipe")?;

        let process = unsafe {
            RawProcess::raw_clone(SIGCHLD | CLONE_NEWNS | CLONE_NEWUSER | CLONE_NEWUTS |
                CLONE_NEWIPC | CLONE_NEWPID | CLONE_NEWNET)
        }.comment_error("Error creating init process for the container")?;

        if process.is_none() {
            container_init_main(
                pipe,
                ContainerInitConfig {
                    daemonize: config.daemonize,
                    cmd: config.cmd.clone(),
                    cmd_args: config.cmd_args.clone(),
                    environment: config.environment,
                    redirect_stderr: config.redirect_stderr,
                    redirect_stdin: config.redirect_stdin,
                    redirect_stdout: config.redirect_stdout,
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
        sudo!("cp", "--recursive"/*, "--one-file-system"*/, "--preserve",
            &self.config.image_path, root_fs.as_str())
            .comment_error("Error copying rootfs")?;
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

    pub fn set_cpu_limit(&mut self) -> Result<()> {
        let perc = self.config.cpu_perc.unwrap_or(100);
        cgroup_create(self.get_id(), perc).comment_error("Error setting up cgroups")?;
        Ok(())
    }

    fn configure_network_with_io_result(&mut self) -> io::Result<()> {
        if let Some(ref conf) = self.config.net {
            let id = &self.get_id().to_string();
            let guest_ip = &conf.cont_addr.to_string();
            let veth_host = &format!("veth{}h", id);
            let veth_guest = &format!("veth{}g", id);

            sudo!("ip", "link", "add", veth_host, "type", "veth", "peer", "name", veth_guest)?;
            sudo!("ip", "link", "set", veth_guest, "netns", id)?;
            sudo!("nsenter", "--net", "-t", id, "ip", "link", "set", "lo", "up")?;
            sudo!("nsenter", "--net", "-t", id, "ip", "link", "set", veth_guest, "name", "eth0")?;
            sudo!("nsenter", "--net", "-t", id, "ip", "link", "set", "eth0", "up")?;
            sudo!("nsenter", "--net", "-t", id, "ip", "addr", "add", guest_ip, "dev", "eth0")?;
            sudo!("nsenter", "--net", "-t", id, "ip", "route", "add", "default", "dev", "eth0")?;
            sudo!("ip", "link", "set", veth_host, "up")?;
            if let Some(host_addr) = conf.host_addr {
                sudo!("ip", "addr", "add", &host_addr.to_string(), "dev", veth_host)?;
                sudo!("ip", "route", "add", guest_ip, "dev", veth_host)?;
            }
            if let Some(bridge) = conf.host_bridge.clone() {
                sudo!("ip", "link", "set", veth_host, "master", &bridge)?;
            }
        }
        Ok(())
    }

    pub fn configure_network(&mut self) -> Result<()> {
        self.configure_network_with_io_result().comment_error("Error configuring network")
    }

    pub fn finish(self) -> Result<Container> {
        // TODO: wait for init process to finish initialization
        Ok(Container {
            process: self.process,
            is_daemon: self.config.daemonize,
        })
    }
}
