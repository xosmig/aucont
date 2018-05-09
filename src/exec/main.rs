extern crate aucont;
#[macro_use]
extern crate clap;
extern crate libc;

use ::aucont::pid_t;
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

    let cont_init_proc = aucont::RawProcess::from_pid(id);
    cont_init_proc.ns_enter("user").expect("Error entering user namespace");
    cont_init_proc.ns_enter("uts").expect("Error entering uts namespace");
    cont_init_proc.ns_enter("net").expect("Error entering net namespace");
    cont_init_proc.ns_enter("ipc").expect("Error entering ipc namespace");
    cont_init_proc.ns_enter("cgroup").expect("Error entering cgroup namespace");
    cont_init_proc.ns_enter("pid").expect("Error entering pid namespace");
    cont_init_proc.ns_enter_mnt().expect("Error entering namespace");

    let mut child = Command::new(cmd)
        .args(cmd_args)
        .spawn()
        .expect("Spawn");
    let exit_status = child.wait()
        .expect("Wait");
    process::exit(match exit_status.code() {
        Some(code) => code,
        None => ::libc::EINTR, // the process is killed by a signal
    })
}
