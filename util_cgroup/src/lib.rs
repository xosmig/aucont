#[macro_use]
extern crate aucont_core as core;

use ::core::raw_process::pid_t;
use ::std::io;
use ::core::{shell, getuid, getgid};
use ::core::aucont_paths::aucont_util;

pub fn cgroup_create(cont_id: pid_t, perc: u32) -> io::Result<()> {
   sudo!(&aucont_util("cgroup"), &cont_id.to_string(), "create",
       "--perc", &perc.to_string(),
       "--uid", &getuid().to_string(),
       "--gid", &getgid().to_string())
}

pub fn cgroup_enter(cont_id: pid_t, target: pid_t) -> io::Result<()> {
   sudo!(&aucont_util("cgroup"), &cont_id.to_string(), "enter",
       "--target", &target.to_string())
}

pub fn cgroup_delete(cont_id: pid_t) -> io::Result<()> {
   sudo!(&aucont_util("cgroup"), &cont_id.to_string(), "delete")
}
