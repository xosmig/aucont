use ::*;
use ::raw_process::*;
use ::std::*;
use ::std::io::Write;
use ::nix::unistd::{getuid, getgid, Uid, Gid};
use super::{Error, Result, Container, CommentError};
use super::container_init_main::*;

pub struct ContainerConfig {
    pub daemonize: bool,
    pub image_path: String,
    pub cmd: String,
    pub cmd_args: Vec<String>,
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
        factory.finish()
    }

    pub fn new(config: ContainerConfig) -> Result<Self> {
        let pipe = Pipe::new().comment_error("ERROR creating pipe")?;

        let process = unsafe {
            RawProcess::raw_clone(SIGCHLD | CLONE_NEWNS | CLONE_NEWUSER |
                CLONE_NEWUTS | CLONE_NEWIPC | CLONE_NEWPID | CLONE_NEWNET)
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
        let root_fs: &str = &container_root_fs(self.get_id());

        let cp = process::Command::new("sudo")
            .args(&["cp", "--recursive", "--one-file-system", "--preserve"])
            .args(&[&self.config.image_path, root_fs])
            .output().comment_error("Cannot copy rootfs")?;

        if !cp.status.success() {
            return Err(Error::simple("ERROR copying the image"));
        }
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

    pub fn finish(mut self) -> Result<Container> {
        // send the pid to the init process
        write!(self.pipe, "{}", self.process.get_pid())
            .comment_error("Internal error (writing PID to pipe)")?;
        // TODO: wait for init process to finish initialization
        Ok(Container {
            process: self.process,
            is_daemon: self.config.daemonize,
        })
    }
}
