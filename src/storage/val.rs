use std::{
    io::{self, Read, Write},
    ops::{Add, Div, Mul, Sub},
};

use super::Typ;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Val {
    String(String),
    Number(i64),
    Boolean(bool),
}

impl Val {
    /// Get the type of this value
    pub fn get_type(&self) -> Typ {
        match self {
            Self::String(_) => Typ::String,
            Self::Number(_) => Typ::Number,
            Self::Boolean(_) => Typ::Boolean,
        }
    }

    /// Write this value in the given writer
    pub fn serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[self.get_type().serialize()])?;
        match self {
            Self::String(s) => {
                writer.write_all(&(s.len() as u8).to_le_bytes())?;
                writer.write_all(s.as_bytes())?;
            }
            Self::Number(i) => writer.write_all(&i.to_le_bytes())?,
            Self::Boolean(i) => writer.write_all(&[if *i { 0 } else { 1 }])?,
        }

        Ok(())
    }

    /// Read a value from a reader
    pub fn deserialize<R: Read>(reader: &mut R) -> io::Result<Val> {
        let mut type_number = [0];
        reader.read_exact(&mut type_number)?;
        let typ = Typ::deserialize(type_number[0]).expect("Unknown type");

        Ok(match typ {
            Typ::Number => {
                let mut buf = [0; 8];
                reader.read_exact(&mut buf)?;
                Val::Number(i64::from_le_bytes(buf))
            }
            Typ::String => {
                let mut len = [0];
                reader.read_exact(&mut len)?;

                let mut buf: Vec<u8> = std::iter::repeat(0).take(len[0].into()).collect();
                reader.read_exact(&mut buf)?;
                Val::String(String::from_utf8(buf).unwrap())
            }
            Typ::Boolean => {
                let mut buf = [0; 1];
                reader.read_exact(&mut buf)?;
                Val::Boolean(buf[0] == 0)
            }
        })
    }

    pub fn to_bool(&self) -> bool {
        if let Val::Boolean(i) = self {
            *i
        } else {
            panic!()
        }
    }
}

impl Add for Val {
    type Output = Val;

    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.get_type(), rhs.get_type());

        match (self, rhs) {
            (Val::String(a), Val::String(b)) => Val::String(format!("{a}{b}")),
            (Val::Number(a), Val::Number(b)) => Val::Number(a + b),
            _ => unreachable!(),
        }
    }
}
impl Sub for Val {
    type Output = Val;

    fn sub(self, rhs: Self) -> Self::Output {
        assert_eq!(self.get_type(), Typ::Number);
        assert_eq!(rhs.get_type(), Typ::Number);

        match (self, rhs) {
            (Val::Number(a), Val::Number(b)) => Val::Number(a - b),
            _ => unreachable!(),
        }
    }
}
impl Mul for Val {
    type Output = Val;

    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(self.get_type(), Typ::Number);
        assert_eq!(rhs.get_type(), Typ::Number);

        match (self, rhs) {
            (Val::Number(a), Val::Number(b)) => Val::Number(a * b),
            _ => unreachable!(),
        }
    }
}
impl Div for Val {
    type Output = Val;

    fn div(self, rhs: Self) -> Self::Output {
        assert_eq!(self.get_type(), Typ::Number);
        assert_eq!(rhs.get_type(), Typ::Number);

        match (self, rhs) {
            (Val::Number(a), Val::Number(b)) => Val::Number(a / b),
            _ => unreachable!(),
        }
    }
}
impl PartialOrd for Val {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        assert_eq!(self.get_type(), rhs.get_type());

        match (self, rhs) {
            (Val::String(a), Val::String(b)) => a.partial_cmp(b),
            (Val::Number(a), Val::Number(b)) => a.partial_cmp(b),
            (Val::Boolean(a), Val::Boolean(b)) => a.partial_cmp(b),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Typ, Val};

    #[test]
    fn ser_de_typ() {
        let typ = Typ::Number;
        assert_eq!(typ, Typ::deserialize(typ.serialize()).unwrap());
    }

    #[test]
    fn ser_val() {
        let val = Val::Number(42);
        let mut buf = vec![];
        val.serialize(&mut buf).unwrap();
        assert_eq!(vec![1, 42, 0, 0, 0, 0, 0, 0, 0], buf);

        let val = Val::String("hello ".to_owned());
        let mut buf = vec![];
        val.serialize(&mut buf).unwrap();
        assert_eq!(vec![0, 6, b'h', b'e', b'l', b'l', b'o', b' '], buf);

        let val = Val::Boolean(true);
        let mut buf = vec![];
        val.serialize(&mut buf).unwrap();
        assert_eq!(vec![2, 0], buf);
    }

    #[test]
    fn de_val() {
        let buf = vec![0, 6, b'h', b'e', b'l', b'l', b'o', b' '];
        let val = Val::deserialize(&mut buf.as_slice()).unwrap();
        assert_eq!(val, Val::String("hello ".into()));

        let buf = vec![1, 42, 0, 0, 0, 0, 0, 0, 0];
        let val = Val::deserialize(&mut buf.as_slice()).unwrap();
        assert_eq!(val, Val::Number(42));

        let buf = vec![2, 0];
        let val = Val::deserialize(&mut buf.as_slice()).unwrap();
        assert_eq!(val, Val::Boolean(true));
    }

    #[test]
    fn ser_de_row() {
        let row = vec![
            Val::String("Hello".into()),
            Val::Number(42),
            Val::Number(-1),
        ];

        let mut buf = vec![];
        for val in &row {
            val.serialize(&mut buf).unwrap();
        }
        println!("{buf:x?}");

        let mut reader = buf.as_slice();
        let mut deserialized = vec![];
        for _ in 0..3 {
            deserialized.push(Val::deserialize(&mut reader).unwrap());
        }

        assert_eq!(row, deserialized);
    }
}
