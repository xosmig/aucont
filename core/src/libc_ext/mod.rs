mod raw {
    use ::libc::c_int;

    extern "C" {
        pub fn get_nprocs() -> c_int;
    }
}

pub fn get_nprocs() -> u32 {
    unsafe { raw::get_nprocs() as u32 }
}
