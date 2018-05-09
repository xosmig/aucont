use ::core::*;
use ::core::redirect_io::*;
use ::core::raw_process::CLONE_NEWCGROUP;
use ::core::check::Check;
use ::std::*;
use ::std::os::unix::process::CommandExt;

pub struct ContainerInitConfig {
    pub cmd: String,
    pub cmd_args: Vec<String>,
    pub daemonize: bool,
}

pub fn container_init_main(mut pipe: Pipe, config: ContainerInitConfig) -> ! {
    let pid_in_host: pid_t = read_number(&mut pipe)
        .check("Internal error (PID from pipe)") as pid_t;

    // cgroup namespace has to be unshared separately when new cgroup roots are established
    sys_unshare(CLONE_NEWCGROUP).check("Unshare cgroup namespace");

    if config.daemonize {
        setsid().check("ERROR daemonizing container");
        let stdin_file: &str = &container_info_file(pid_in_host, "stdin");
        fs::File::create(&stdin_file)
            .check("ERROR creating stdin file");
        redirect_stdin(stdin_file)
            .check("ERROR redirecting stdin");
        redirect_stdout(container_info_file(pid_in_host, "stdout"))
            .check("ERROR redirecting stdout");
        redirect_stderr(container_info_file(pid_in_host, "stderr"))
            .check("ERROR redirecting stderr");
    }
    sethostname("container").check("ERROR setting hostname");

    let root_fs: &str = &container_root_fs(pid_in_host);
    let old_root: &str = &format!("{}/mnt", root_fs);

    sys_mount(root_fs, root_fs, "ignored", MS_BIND | MS_REC, None)
        .check("Internal error (bind rootfs)");
    chdir(root_fs).check("Internal error (chdir)");
    pivot_root(".", old_root).check("Internal error (pivot_root)");
    chroot(".").check("Internal error (chroot)");

    sys_mount("procfs", "/proc/", "proc", 0, None).check("ERROR mounting procfs");
    sys_mount("sysfs", "/sys/", "sysfs", 0, None).check("ERROR mounting sysfs");
    umount2("/mnt", MntFlags::MNT_DETACH).check("ERROR unmounting old root");

    // either returns an error or doesn't return at all
    let err = process::Command::new(&config.cmd).args(config.cmd_args).exec();

    panic!("Error starting the process '{}': {}", config.cmd, err);
}
