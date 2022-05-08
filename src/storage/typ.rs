#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Typ {
    String = 0,
    Number = 1,
}

impl Typ {
    pub fn serialize(&self) -> u8 {
        *self as u8
    }

    pub fn deserialize(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::String),
            1 => Some(Self::Number),
            _ => None,
        }
    }
}
