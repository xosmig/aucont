extern crate libc;
extern crate num;
extern crate nix;

mod pipe;
mod libc_wrappers;
mod utils;
pub mod redirect_io;
pub mod raw_process;
pub mod container;
pub mod sys_return;

pub use self::pipe::*;
pub use self::raw_process::*;
pub use self::libc_wrappers::*;
pub use self::utils::*;

pub const CONTAINERS_DIR: &'static str = "/tmp/aucont/containers";

fn container_dir_suf(cont_pid: pid_t, suf: &str) -> String {
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
