extern crate aucont;
extern crate clap;
extern crate nix;

mod container_init_main;
mod container_factory;
use ::container_factory::*;
use ::std::*;

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

    let mut factory = ContainerFactory::new(matches);

    factory.map_uid();
    factory.init_dir();
    factory.record_info();

    let container = factory.finish();

    println!("{}", container.get_id());

    if !container.is_daemon() {
        let ret = container.wait().expect("Internal error (join)");
        process::exit(ret);
    }
}
