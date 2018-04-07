use ::*;
use ::std::*;
use ::std::os::unix::process::CommandExt;
use ::nix::unistd::{pivot_root, chroot, chdir, sethostname};
use ::nix::mount::{MntFlags, umount2};


pub struct ContainerInitConfig<'a> {
    pub cmd: &'a str,
    pub cmd_args: Vec<&'a str>,
}

pub fn container_init_main(mut pipe: Pipe, config: ContainerInitConfig) -> ! {
    let pid_in_host: pid_t = read_number(&mut pipe)
        .expect("Internal error (PID from pipe)") as pid_t;

    sethostname("container").expect("ERROR setting hostname");

    let root_fs: &str = &container_root_fs(pid_in_host);
    let old_root: &str = &format!("{}/mnt", root_fs);

    sys_mount(root_fs, root_fs, "ignored", MS_BIND | MS_REC).expect("Internal error (bind rootfs)");
    chdir(root_fs).expect("Internal error (chdir)");
    pivot_root(".", old_root).expect("Internal error (pivot_root)");
    chroot(".").expect("Internal error (chroot)");

    sys_mount("procfs", "/proc/", "proc", 0).expect("ERROR mounting procfs");
    sys_mount("sysfs", "/sys/", "sysfs", 0).expect("ERROR mounting sysfs");
    umount2("/mnt", MntFlags::MNT_DETACH).expect("ERROR unmounting old root");

    // either returns an error or doesn't return at all
    let err = process::Command::new(config.cmd).args(config.cmd_args).exec();

    panic!("Error starting the process '{}': {}", config.cmd, err);
}
