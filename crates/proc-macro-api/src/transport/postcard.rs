//! Postcard encode and decode implementations.

use std::io::{self, BufRead, Write};

use serde::{Serialize, de::DeserializeOwned};

pub fn read<'a, R: BufRead + ?Sized>(
    inp: &mut R,
    buf: &'a mut Vec<u8>,
) -> io::Result<Option<&'a mut Vec<u8>>> {
    buf.clear();
    let n = inp.read_until(0, buf)?;
    if n == 0 {
        return Ok(None);
    }
    Ok(Some(buf))
}

pub fn write<W: Write + ?Sized>(out: &mut W, buf: &[u8]) -> io::Result<()> {
    out.write_all(buf)?;
    out.flush()
}

pub fn encode<T: Serialize>(msg: &T) -> io::Result<Vec<u8>> {
    postcard::to_allocvec_cobs(msg).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn decode<T: DeserializeOwned>(buf: &mut [u8]) -> io::Result<T> {
    postcard::from_bytes_cobs(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
