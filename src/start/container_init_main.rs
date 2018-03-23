use ::aucont::*;
use ::std::*;
use ::std::io::{Read, Write};
use ::std::os::unix::process::CommandExt;
use ::nix::unistd::{pivot_root, chroot, chdir, sethostname};
use ::nix::mount::{MntFlags, umount2};


pub struct ContainerInitConfig<'a> {
    pub image_path: &'a str,
    pub cmd: &'a str,
    pub cmd_args: Vec<&'a str>,
}

pub fn container_init_main(pipe: &mut Pipe, config: ContainerInitConfig) -> ! {
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

    let cp = process::Command::new("cp").arg("-rx")
        .arg(config.image_path)
        .arg(root_fs)
        .output().expect("Cannot copy the image");
    if !cp.status.success() {
        eprint!("ERROR copying the image: ");
        io::stderr().write(&cp.stderr).unwrap();
        process::exit(1);
    }

    sethostname("container").expect("ERROR setting hostname");

    sys_mount(root_fs, root_fs, "ignored", MS_BIND | MS_REC).expect("Internal error (bind rootfs)");
    let old_root: &str = &format!("{}/mnt", root_fs);
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
