use ::std::fmt::Display;

pub trait Check<T> {
    // like expect, but uses Display instead of Debug
    fn check<S: AsRef<str>>(self, msg: S) -> T;

    // Prints the error to the error stream and continues execution.
    // Used in destructors and deferred functions to avoid panics while already unwinding.
    fn log_error<S: AsRef<str>>(self, msg: S) -> Option<T>;
}

impl<E: Display, T> Check<T> for Result<T, E> {
    fn check<S: AsRef<str>>(self, msg: S) -> T {
        match self {
            Ok(t) => t,
            Err(e) => panic!("{}: {}", msg.as_ref(), e),
        }
    }

    fn log_error<S: AsRef<str>>(self, msg: S) -> Option<T> {
        match self {
            Ok(t) => Some(t),
            Err(e) => { eprintln!("{}: {}", msg.as_ref(), e); None },
        }
    }
}
