extern crate aucont;

use std::fs;

fn main() {
    let dir = fs::read_dir("/tmp/aucont/containers")
        .expect("Error opening directory");

    for entry in dir {
        let path = entry.expect("Error listing directory").path();
        let cont_name_path = path.iter().last().expect("Error reading container id");
        let cont_name_str = cont_name_path.to_str().expect("Error reading container id");
        println!("{}", cont_name_str);
    }
}
