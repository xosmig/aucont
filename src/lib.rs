extern crate libc;
extern crate num;

mod sys_return;
mod libc_wrappers;
pub use libc_wrappers::*;

pub fn container_dir(cont_pid: pid_t, suf: &str) -> String {
    format!("/tmp/aucont/containers/{}/{}", cont_pid, suf)
}

pub fn container_root_fs(cont_pid: pid_t) -> String {
    container_dir(cont_pid, "rootfs")
}
