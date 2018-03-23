extern crate libc;
extern crate num;
extern crate nix;

mod sys_return;
mod pipe;
pub use pipe::*;
mod process;
pub use process::*;
mod libc_wrappers;
pub use libc_wrappers::*;
mod container;
pub use container::*;
mod utils;
pub use utils::*;

fn container_dir_suf(cont_pid: pid_t, suf: &str) -> String {
    format!("/tmp/aucont/containers/{}{}", cont_pid, suf)
}

pub fn container_dir(cont_pid: pid_t) -> String {
    container_dir_suf(cont_pid, "")
}

pub fn container_info_dir(cont_pid: pid_t) -> String {
    container_dir_suf(cont_pid, "/info")
}

pub fn container_root_fs(cont_pid: pid_t) -> String {
    container_dir_suf(cont_pid, "/rootfs")
}
