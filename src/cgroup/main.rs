extern crate aucont;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate scopeguard;

use ::std::fs;
use ::std::path::Path;
use ::aucont::{pid_t, container_dir_suf, sys_mount, sys_umount, sys_chown,
               uid_t, gid_t, get_nprocs};
use ::std::io::Write;

fn main() {
    let matches = clap::App::new("aucont_util_cgroup")
        .version("0.1")
        .about("Utility tool used by aucont to manage cgroups. Requires CAP_SYS_ADMIN.")
        .setting(clap::AppSettings::AllowLeadingHyphen)
        .arg(clap::Arg::with_name("pid")
            .index(1)
            .required(true)
            .value_name("ID")
            .help("Container id as returned by aucont_start"))
        .arg(clap::Arg::with_name("uid")
            .index(2)
            .required_unless("del")
            .value_name("UID"))
        .arg(clap::Arg::with_name("gid")
            .index(3)
            .required_unless("del")
            .value_name("GID"))
        .arg(clap::Arg::with_name("add")
            .long("add")
            .value_name("CPU_PERC")
            .required_unless("del")
            .conflicts_with("del")
            .help("Add the container <ID> to a cpu cgroup with cpu restricted to <CPU_PERC>"))
        .arg(clap::Arg::with_name("del")
            .long("del")
            .required_unless("add")
            .help("Delete the container's cgroup"))
        .get_matches();

    let id = value_t_or_exit!(matches.value_of("pid"), pid_t);
    let id_str = &id.to_string() as &str;

    let mount_path = &container_dir_suf(id, "/cpu_cgroup") as &str;
    let cgroup_path = &format!("{}/aucont_{}", mount_path, id) as &str;

    fs::create_dir(mount_path)
        .expect("Error creating directory for cgroup mount");
    defer! {{ fs::remove_dir(mount_path).expect("Error removing cgroup directory") }}
    sys_mount("aucont_cpu_cgroup", mount_path, "cgroup", 0, Some("cpu,cpuacct"))
        .expect("Error mounting cgroup filesystem");
    defer! {{ sys_umount(mount_path).expect("Error unmounting cgroup"); }}

    if matches.is_present("add") {
        let perc = value_t_or_exit!(matches.value_of("add"), u32);
        let uid = value_t_or_exit!(matches.value_of("uid"), uid_t);
        let gid = value_t_or_exit!(matches.value_of("gid"), gid_t);

        if perc > 100 {
            panic!("Percent of cpu must not be greater than 100");
        }

        fs::create_dir(cgroup_path).expect("Error creating cgroup");
        // TODO: defer_on_unwind! {{ }}
        sys_chown(cgroup_path, uid, gid)
            .expect("Error setting owner of cgroup");
        fs::File::create(format!("{}/tasks", cgroup_path))
            .and_then(|mut f| f.write_all(id_str.as_bytes()))
            .expect("Error adding process to cgroup");

        let nprocs = get_nprocs();
        let period = 100000;
        let quota = period * perc * nprocs / 100;

        fs::File::create(format!("{}/cpu.cfs_period_us", cgroup_path))
            .and_then(|mut f| f.write_all(period.to_string().as_bytes()))
            .expect("Error setting cgroup cpu period");

        fs::File::create(format!("{}/cpu.cfs_quota_us", cgroup_path))
            .and_then(|mut f| f.write_all(quota.to_string().as_bytes()))
            .expect("Error setting cgroup cpu quota");
    }

    if matches.is_present("del") {
        fn remove_cgroup_rec<P: AsRef<Path>>(path: P) {
            for entry in fs::read_dir(path.as_ref()).expect("Error opening cgroup dir") {
                let path = entry.expect("Error accessing cgroup data").path();
                if path.is_dir() {
                    remove_cgroup_rec(path);
                }
            }
            fs::remove_dir(path).expect("Error deleting cgroup");
        }
        if Path::new(cgroup_path).exists() {
            remove_cgroup_rec(cgroup_path);
        }
    }
}