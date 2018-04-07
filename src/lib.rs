extern crate libc;
extern crate num;
extern crate nix;

mod sys_return;
mod pipe;
mod libc_wrappers;
mod utils;
pub mod raw_process;
pub mod container;

pub use self::pipe::*;
pub use self::raw_process::*;
pub use self::libc_wrappers::*;
pub use self::utils::*;

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
