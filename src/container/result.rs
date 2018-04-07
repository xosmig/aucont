use std::{string::String, error, fmt, result};

pub struct Error {
    comment: String,
    cause: Option<Box<error::Error>>,
}

impl Error {
    pub fn new<S, E>(comment: S, cause: E) -> Error
        where S: Into<String>, E: Into<Box<error::Error>> {
        Error { comment: comment.into(), cause: Some(cause.into()) }
    }

    pub fn simple<S: Into<String>>(error_message: S) -> Error {
        Error { comment: error_message.into(), cause: None }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.cause {
            &Some(ref cause) => {
                if self.comment == "" {
                    write!(f, "{}", cause)
                } else {
                    write!(f, "{}: {}", self.comment, cause)
                }
            }
            &None => {
                write!(f, "{}", self.comment)
            }
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Container error"
    }

    fn cause(&self) -> Option<&error::Error> {
        self.cause.as_ref().map(|e| e.as_ref())
    }
}

impl From<::std::io::Error> for Error {
    fn from(error: ::std::io::Error) -> Error {
        Error::new("", error)
    }
}

pub type Result<T> = result::Result<T, Error>;

pub trait CommentError<T> {
    fn comment<S: Into<String>>(self, comment: S) -> Result<T>;
}

impl<T, E: Into<Box<error::Error>>> CommentError<T> for result::Result<T, E> {
    fn comment<S: Into<String>>(self, comment: S) -> Result<T> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(Error::new(comment, e))
        }
    }
}
