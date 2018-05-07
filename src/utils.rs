use ::std::{io, fs, str};
use ::std::io::ErrorKind::InvalidData;

pub fn read_number<R: io::Read>(input: &mut R) -> io::Result<i64> {
    let mut buf = vec![0 as u8; 20];
    let read = input.read(&mut buf)?;

    let str: &str = str::from_utf8(&buf[0..read])
        .or_else(|e| Err(io::Error::new(InvalidData, e)))?
        .trim();
    str.parse()
        .or_else(|e| Err(io::Error::new(InvalidData, e)))
}

pub fn list_dir_files<S: AsRef<str>>(dir_path: S) -> io::Result<Vec<String>> {
    let dir = fs::read_dir(dir_path.as_ref())?;
    let mut res = vec![];

    for entry in dir {
        let path = entry?.path();
        let file_name_path = path.iter().last().unwrap();
        let file_name_str = file_name_path.to_str()
            .ok_or(io::Error::new(InvalidData, "Something wrong with the filename"))?;
        res.push(file_name_str.to_string());
    }

    Ok(res)
}
