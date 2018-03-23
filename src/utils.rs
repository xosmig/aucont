use ::std::io::*;
use ::std::str::*;
use ::std::io::ErrorKind::InvalidData;

pub fn read_number<R: Read>(input: &mut R) -> Result<i64> {
    let mut buf = vec![0 as u8; 20];
    let read = input.read(&mut buf)?;

    let str: &str = from_utf8(&buf[0..read])
        .or_else(|e| Err(Error::new(InvalidData, e)))?
        .trim();
    str.parse()
        .or_else(|e| Err(Error::new(InvalidData, e)))
}
