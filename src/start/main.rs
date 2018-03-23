extern crate aucont;
extern crate clap;
extern crate nix;

use aucont::*;
use std::*;
use std::ffi::CString;
use std::os::unix::process::CommandExt;
use std::io::{Write, Read};
use nix::unistd::{pivot_root, chroot, chdir, sethostname, getuid, getgid, Uid, Gid};
use nix::mount::{mount, MsFlags, MntFlags, umount2};

fn container_init_main(matches: clap::ArgMatches, mut pipe: Pipe) -> ! {
    let pid_in_host: pid_t = {
        let mut buf = vec![0 as u8; 20];
        const ERR: &'static str = "Internal error (PID from pipe)";
        let read = pipe.read(&mut buf).expect(ERR);
        str::from_utf8(&buf[0..read]).expect(ERR).parse().expect(ERR)
    };

    let container_dir: &str = &container_dir(pid_in_host);
    let root_fs: &str = &container_root_fs(pid_in_host);

    if path::Path::new(container_dir).exists() {
        eprintln!("Internal error ('{}' already exists)", root_fs);
        process::exit(1);
    }
    fs::create_dir_all(container_dir)
        .expect("Internal error (create rootfs dir)");

    let cp = process::Command::new("cp")
        .arg("-rx")
        .arg(matches.value_of("image_path").unwrap())
        .arg(root_fs)
        .output()
        .expect("Cannot copy the image");
    if !cp.status.success() {
        eprint!("ERROR copying the image: ");
        io::stderr().write(&cp.stderr).unwrap();
        process::exit(1);
    }

    sethostname("container").expect("ERROR setting hostname");

    sys_mount(root_fs, root_fs, "ignored", MS_BIND | MS_REC).expect("Internal error (bind rootfs)");
    let old_root: &str = &format!("{}/mnt", root_fs);
    pivot_root(root_fs, old_root).expect("Internal error (pivot_root)");
    chroot("/").expect("Internal error (chroot)");
    chdir("/").expect("Internal error (chdir)");

    // FIXME: some problems here: can't swap the lines
    sys_mount("proc", "/proc/", "proc", 0).expect("ERROR mounting proc fs");
    umount2("/mnt", MntFlags::MNT_DETACH).expect("ERROR unmounting old root");

    let cmd = matches.value_of("cmd").unwrap();
    let mut command = process::Command::new(cmd);
    if let Some(args) = matches.values_of("cmd_args") {
        command.args(args);
    }
    // either returns an error or doesn't return at all
    let err = command.exec();

    panic!("Error starting the process '{}': {}", cmd, err);
}

fn main() {
    let matches = clap::App::new("aucont_start")
        .version("0.1")
        .about("Starts a new container. \
        Prints the ID of the started container to the standard output.")
        .arg(clap::Arg::with_name("cpu")
            .long("cpu")
            .takes_value(true)
            .value_name("CPU_PERC")
            .help("Percent of cpu resources allocated for container (0..100)."))
        .arg(clap::Arg::with_name("net")
            .long("net")
            .takes_value(true)
            .value_name("IP")
            .help("Create virtual network between host and container.\n\
            IP — container ip address, IP+1 — host ip address."))
        .arg(clap::Arg::with_name("daemonize")
            .short("d")
            .long("daemonize")
            .help("Start the process as a daemon."))
        .arg(clap::Arg::with_name("image_path")
            .index(1)
            .required(true)
            .value_name("IMAGE_PATH")
            .help("Path to the image of the container file system."))
        .arg(clap::Arg::with_name("cmd")
            .index(2)
            .required(true)
            .value_name("CMD")
            .help("Command to run inside container."))
        .arg(clap::Arg::with_name("cmd_args")
            .index(3)
            .multiple(true)
            .required(false)
            .value_name("ARGS")
            .help("Arguments for <cmd>."))
        .get_matches();

    let mut pipe = Pipe::new().expect("ERROR creating pipe");

    let process = unsafe {
        Process::raw_clone(SIGCHLD | CLONE_NEWNS | CLONE_NEWUSER | CLONE_NEWUTS |
            CLONE_NEWIPC | CLONE_NEWPID | CLONE_NEWNET)
    }.expect("Error creating init process for the container");

    match process {
        None => container_init_main(matches, pipe),
        Some(mut process) => {
            write!(pipe, "{}", process.get_pid()).expect("Internal error (writing PID to pipe)");

            let container_dir: &str = &container_dir(process.get_pid());
            process.uid_map()
                .entry(getuid(), Uid::from_raw(0))
                .set()
                .expect("Internal error: cannot set UID mapping");

            process.gid_map()
                .entry(getgid(), Gid::from_raw(0))
                .set()
                .expect("Internal error: cannot set GID mapping");

            // tell the container its ID and start the init process
            println!("{}", process.get_pid());

            let ret_code = process.wait().expect("Internal error (waiting init process to end)");

            fs::remove_dir_all(container_dir)
                .expect("Internal error (removing root_fs of finished container)");

            process::exit(ret_code);
        }
    }
}
