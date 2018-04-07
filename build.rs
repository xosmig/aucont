extern crate gcc;

fn main() {
    gcc::Build::new()
        .file("src/redirect_io/lib.c")
        .compile("redirect_io");
}
