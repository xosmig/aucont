extern crate aucont;
#[macro_use]
extern crate clap;

use ::std::*;
use ::aucont::*;

fn main() {
    let matches = clap::App::new("aucont_start")
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
    let mut container = Container::new(id).expect("Error accessing container");
    container.cancel(signal).expect("Error cancelling container");
    let ret = container.wait().expect("Internal error (join)");
    process::exit(ret);
}
