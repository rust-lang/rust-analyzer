//! Implement how to encoding and decoding new protocol message on stdio stream
//!
//! Current implementation encoded message to | message length | message content |

use serde::de::DeserializeOwned;
use std::io::{self, BufRead, Read, Write};

use super::msg::{C2SMsg, S2CMsg};

fn read_usize_be<R: Read + ?Sized>(reader: &mut R) -> io::Result<usize> {
    let mut buf = [0u8; std::mem::size_of::<usize>()];
    reader.read_exact(&mut buf)?;
    Ok(usize::from_be_bytes(buf))
}

fn write_usize_be<W: Write + ?Sized>(writer: &mut W, value: usize) -> io::Result<()> {
    writer.write_all(&value.to_be_bytes()) // Convert and write as Big-Endian
}

pub trait ProtoPostcard: serde::Serialize + DeserializeOwned {
    fn receive_proto<R: BufRead + ?Sized>(reader: &mut R) -> io::Result<Self> {
        let msg_len = read_usize_be(reader)? as usize;
        let mut buf = vec![0u8; msg_len];

        reader.read_exact(&mut buf)?;
        postcard::from_bytes::<Self>(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn send_proto<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        let bytes = postcard::to_allocvec(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        write_usize_be(writer, bytes.len())?;
        writer.write_all(&bytes)?;
        writer.flush()
    }
}

// NOTE: With default implementation, C2SMsg and S2CMsg can both be send through stdio
impl ProtoPostcard for C2SMsg {}
impl ProtoPostcard for S2CMsg {}
