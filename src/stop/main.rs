extern crate aucont;
extern crate clap;

//use ::aucont::*;

fn main() {
    let _matches = clap::App::new("aucont_start")
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


}
