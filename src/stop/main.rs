extern crate aucont;
#[macro_use]
extern crate clap;

use ::std::*;
use ::aucont::*;
use ::aucont::check::Check;
use ::container::Container;

fn main() {
    let matches = clap::App::new("aucont_stop")
        .version("0.1")
        .about("Stop a daemonized container, started by aucont_start.")
        .arg(clap::Arg::with_name("pid")
            .index(1)
            .required(true)
            .value_name("ID")
            .help("Container id as returned by aucont_start."))
        .arg(clap::Arg::with_name("sig_num")
            .index(2)
            .required(false)
            .value_name("SIG_NUM")
            .default_value("15")
            .help("Number of the signal sent to the container process."))
        .get_matches();

    let id = value_t_or_exit!(matches.value_of("pid"), pid_t);
    let signal = value_t_or_exit!(matches.value_of("sig_num"), c_int);
    let mut container = Container::connect(id).check("Error accessing container");
    if !container.is_daemon() {
        panic!("Contained with id '{}' is not a daemon", container.get_id());
    }

    container.cancel(signal).check("Error cancelling container");
    let ret = container.wait_and_clear().check("Internal error (join)");
    process::exit(ret);
}
