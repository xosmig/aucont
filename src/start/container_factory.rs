use ::aucont::*;
use ::std::*;
use ::std::io::Write;
use ::clap;
use ::nix::unistd::{getuid, getgid, Uid, Gid};
use ::container_init_main::*;

pub struct ContainerFactory {
    is_daemon: bool,
    process: Process,
    pipe: Pipe,
}

impl ContainerFactory {
    pub fn new(config: clap::ArgMatches) -> Self {
        let pipe = Pipe::new().expect("ERROR creating pipe");

        let process = unsafe {
            Process::raw_clone(SIGCHLD | CLONE_NEWNS | CLONE_NEWUSER | CLONE_NEWUTS |
                CLONE_NEWIPC | CLONE_NEWPID | CLONE_NEWNET)
        }.expect("Error creating init process for the container");

        if process.is_none() {
            container_init_main(
                pipe,
                ContainerInitConfig {
                    image_path: config.value_of("image_path").unwrap(),
                    cmd: config.value_of("cmd").unwrap(),
                    cmd_args: match config.values_of("cmd_args") {
                        Some(args) => args.collect(),
                        None => vec![],
                    },
                },
            );
            // unreachable
        }

        // parent process
        ContainerFactory {
            is_daemon: config.is_present("daemonize"),
            process: process.unwrap(),
            pipe,
        }
    }

    pub fn get_id(&self) -> pid_t {
        self.process.get_pid()
    }

    pub fn map_uid(&mut self) {
        self.process.uid_map()
            .entry(getuid(), Uid::from_raw(0))
            .set().expect("Internal error: cannot set UID mapping");

        self.process.gid_map()
            .entry(getgid(), Gid::from_raw(0))
            .set().expect("Internal error: cannot set GID mapping");
    }

    pub fn init_dir(&mut self) {
        let dir: &str = &container_dir(self.get_id());

        if path::Path::new(dir).exists() {
            eprintln!("Internal error ('{}' already exists)", dir);
            process::exit(1);
        }

        fs::create_dir_all(dir).expect("Internal error (create container dir)");
    }

    pub fn record_info(&mut self) {
        let info_dir = &container_info_dir(self.get_id());
        fs::create_dir_all(info_dir).expect("Internal error (create info dir)");

        let mut daemon_file = fs::File::create(&format!("{}/daemon", info_dir))
            .expect("Internal error (open daemon file)");
        writeln!(daemon_file, "{}", if self.is_daemon { 1 } else { 0 })
            .expect("Internal error (write daemon file)");
    }

    pub fn finish(mut self) -> Container {
        // send the pid to the init process
        write!(self.pipe, "{}", self.process.get_pid())
            .expect("Internal error (writing PID to pipe)");
        // TODO: wait for init process to finish initialization
        unsafe { Container::new(self.get_id()) }
            .expect("Internal error (finish container factory)")
    }
}
