extern crate aucont;
extern crate clap;
extern crate nix;

mod container_init_main;

use ::aucont::*;
use ::std::*;
use ::std::io::Write;
use ::nix::unistd::{getuid, getgid, Uid, Gid};
use ::container_init_main::*;


fn main() {
    let matches = clap::App::new("aucont_start")
        .version("0.1")
        .about("Start a new container. \
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

    if process.is_none() {
        container_init_main(
            &mut pipe,
            ContainerInitConfig {
                image_path: matches.value_of("image_path").unwrap(),
                cmd: matches.value_of("cmd").unwrap(),
                cmd_args: match matches.values_of("cmd_args") {
                    Some(args) => args.collect(),
                    None => vec![],
                },
            },
        );
    }

    let mut process = process.unwrap();

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
    write!(pipe, "{}", process.get_pid()).expect("Internal error (writing PID to pipe)");

    // TODO: wait for a confirmation from the container

    // output the container id. Creation of the container must be finished at this moment
    println!("{}", process.get_pid());

    if !matches.is_present("daemonize") {
        let ret_code = process.wait().expect("Internal error (waiting init process to end)");
        fs::remove_dir_all(container_dir)
            .expect("Internal error (removing root_fs of finished container)");
        process::exit(ret_code);
    }
}
