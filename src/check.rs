use ::std::fmt::Display;

pub trait Check<T> {
    // like expect, but uses Display instead
    fn check<S: AsRef<str>>(self, msg: S) -> T;
}

impl<E: Display, T> Check<T> for Result<T, E> {
    fn check<S: AsRef<str>>(self, msg: S) -> T {
        match self {
            Ok(t) => t,
            Err(e) => panic!("{}: {}", msg.as_ref(), e),
        }
    }
}
