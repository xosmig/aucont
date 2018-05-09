extern crate aucont_core as core;
extern crate aucont_util_cgroup as cgroup;
#[macro_use]
extern crate clap;

use ::core::{pid_t, getpid};
use ::core::libc_wrappers::EINTR;
use ::core::check::Check;
use ::std::process::Command;
use ::std::process;


fn main() {
    let matches = clap::App::new("aucont_exec")
        .version("0.1")
        .about("Start command <CMD> with arguments <ARGS> inside running container with id <ID>. \
        The command is started in interactive mode. \
        Returns the exit code returned by <CMD>.")
        .setting(clap::AppSettings::AllowLeadingHyphen)
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

    // Note: this must be done before entering container namespaces
    cgroup::cgroup_enter(id, getpid()).check("Error entering cgroup of the container");

    let cont_init_proc = ::core::RawProcess::from_pid(id);
    cont_init_proc.ns_enter("user").check("Error entering user namespace");
    cont_init_proc.ns_enter("uts").check("Error entering uts namespace");
    cont_init_proc.ns_enter("net").check("Error entering net namespace");
    cont_init_proc.ns_enter("ipc").check("Error entering ipc namespace");
    cont_init_proc.ns_enter("cgroup").check("Error entering cgroup namespace");
    cont_init_proc.ns_enter("pid").check("Error entering pid namespace");
    cont_init_proc.ns_enter_mnt().check("Error entering mount namespace");

    let mut child = Command::new(cmd).args(cmd_args)
        .spawn().check("Spawn");
    let exit_status = child.wait().check("Wait");
    process::exit(match exit_status.code() {
        Some(code) => code,
        None => EINTR, // the process is killed by a signal
    })
}
