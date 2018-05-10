extern crate aucont_core as core;
extern crate aucont_lib_container as container;
#[macro_use]
extern crate clap;

use ::core::check::Check;
use ::container::factory::*;
use ::std::process;
use ::std::net::Ipv4Addr;

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
            .help("Arguments for <CMD>."))
        .get_matches();

    let net_config = matches.value_of("net").map(|addr_str| {
        let cont_addr: Ipv4Addr = addr_str.parse().check("Can't parse ip address");
        let octets = cont_addr.octets();
        let host_addr = Some(Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3] + 1));
        NetworkConfig { cont_addr, host_addr, host_bridge: None }
    });

    let container = ContainerFactory::new_container(
        ContainerConfig {
            daemonize: matches.is_present("daemonize"),
            image_path: matches.value_of("image_path").unwrap().to_string(),
            cmd: matches.value_of("cmd").unwrap().to_string(),
            cmd_args: match matches.values_of("cmd_args") {
                Some(args) => args.map(|s| s.to_string()).collect(),
                None => vec![],
            },
            net: net_config,
            cpu_perc: matches.value_of("cpu")
                .map(|_| value_t_or_exit!(matches.value_of("cpu"), u32)),
            ..Default::default()
        }
    ).check("ERROR creating container");

    println!("{}", container.get_id());

    if !container.is_daemon() {
        let ret = container.wait_and_clear().check("Internal error (join)");
        process::exit(ret);
    }
}
