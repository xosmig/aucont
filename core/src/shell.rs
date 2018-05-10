use ::std::{self, io, process};
use ::std::ffi::OsStr;

pub fn shell_cmd<I, S1, S2>(cmd: S1, args: I) -> io::Result<()>
    where I: IntoIterator<Item=S2> + std::fmt::Debug, S1: AsRef<str>, S2: AsRef<OsStr>
{
    let error_string = format!("Error executing '{}' with arguments {:?}", cmd.as_ref(), args);

    let output = process::Command::new(cmd.as_ref())
        .args(args)
        .output()?;

    if !output.status.success() {
        let stderr_string = std::str::from_utf8(&output.stderr).unwrap();
        let stdout_string = std::str::from_utf8(&output.stdout).unwrap();
        let error_message = format!("{}\nstderr:\n{}\nstdout:\n{}\n=======",
                                    error_string.as_str(), stderr_string, stdout_string);
        return Err(io::Error::new(io::ErrorKind::Other, error_message));
    }

    Ok(())
}

#[macro_export]
macro_rules! shell {
    ( $cmd: expr, $( $x: expr ),* ) => {
        $crate::shell::shell_cmd($cmd, &[ $( $x ),* ])
    };
}

#[macro_export]
macro_rules! sudo {
    ( $( $x: expr ),* ) => {
        shell!("sudo", $( $x ),*)
    };
}
