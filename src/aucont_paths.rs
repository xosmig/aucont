use ::raw_process::pid_t;


pub const CONTAINERS_DIR: &'static str = "/tmp/aucont/containers";

pub fn container_dir_suf(cont_pid: pid_t, suf: &str) -> String {
    format!("{}/{}{}", CONTAINERS_DIR, cont_pid, suf)
}

pub fn container_dir(cont_pid: pid_t) -> String {
    container_dir_suf(cont_pid, "")
}

pub fn container_info_dir(cont_pid: pid_t) -> String {
    container_dir_suf(cont_pid, "/info")
}

pub fn container_info_file(cont_pid: pid_t, name: &str) -> String {
    container_dir_suf(cont_pid, &format!("/info/{}", name))
}

pub fn container_root_fs(cont_pid: pid_t) -> String {
    container_dir_suf(cont_pid, "/rootfs")
}

pub fn aucont_util(util_name: &str) -> String {
    let this_exe = ::std::env::current_exe().unwrap();
    let exe_dir = this_exe.parent().unwrap();
    let util_exe = exe_dir.join(format!("aucont_util_{}", util_name));
    util_exe.to_str().unwrap().to_string()
}
