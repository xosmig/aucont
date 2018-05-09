extern crate gcc;

fn main() {
    gcc::Build::new()
        .file("src/redirect_io/lib.c")
        .compile("redirect_io");
    gcc::Build::new()
        .file("src/libc_ext/lib.c")
        .compile("libc_ext");
}
