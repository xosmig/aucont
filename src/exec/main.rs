extern crate aucont;
#[macro_use]
extern crate clap;
extern crate libc;

use ::aucont::pid_t;
use ::std::process::Command;
use ::std::os::unix::process::CommandExt;

fn main() {
    let matches = clap::App::new("aucont_start")
        .version("0.1")
        .about("Start command <CMD> with arguments <ARGS> inside running container with id <ID>. \
        The command is started in interactive mode. \
        Returns the exit code returned by <CMD>.")
        .arg(clap::Arg::with_name("pid")
            .index(1)
            .required(true)
            .value_name("ID")
            .help("Container id as returned by aucont_start"))
        .arg(clap::Arg::with_name("cmd")
            .index(2)
            .required(true)
            .value_name("CMD")
            .help("Command to run inside container"))
        .arg(clap::Arg::with_name("cmd_args")
            .index(3)
            .multiple(true)
            .required(false)
            .value_name("ARGS")
            .help("Arguments for <CMD>."))
        .get_matches();

    let id = value_t_or_exit!(matches.value_of("pid"), pid_t);
    let cmd = matches.value_of("cmd").unwrap().to_string();
    let cmd_args = match matches.values_of("cmd_args") {
        Some(args) => args.map(|s| s.to_string()).collect(),
        None => vec![],
    };

    let init_in_host = aucont::RawProcess::from_pid(id)
        .expect("Error accessing container init process");
    init_in_host.ns_enter("mnt").expect("Error entering namespace");

    let init_in_guest = aucont::RawProcess::from_pid(1)
        .expect("Error accessing container init process");

    init_in_guest.ns_enter("uts").expect("Error entering namespace");
    init_in_guest.ns_enter("net").expect("Error entering namespace");
    init_in_guest.ns_enter("ipc").expect("Error entering namespace");
    init_in_guest.ns_enter("cgroup").expect("Error entering namespace");
    init_in_guest.ns_enter("pid").expect("Error entering namespace");
    init_in_guest.ns_enter("user").expect("Error entering namespace");

    unsafe {
        ::aucont::sys_return::sys_return_unit(::libc::setuid(0))
            .expect("Error setting uid");
        ::aucont::sys_return::sys_return_unit(::libc::setgid(0))
            .expect("Error setting gid");
    }


    Command::new(cmd)
        .args(cmd_args)
        .exec();
}
