use std::io::{self, Write};

use sled::IVec;

pub fn serialize_ivec<W: Write>(ivec: &IVec, writer: &mut W) -> io::Result<()> {
    writer.write_all(&(ivec.len() as u8).to_le_bytes())?;
    writer.write_all(ivec.as_ref())?;

    Ok(())
}

pub fn display_ivec(ivec: &IVec) -> String {
    String::from_utf8(ivec.as_ref().to_vec()).unwrap()
}
