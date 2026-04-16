#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadDeviceIdCode {
    Basic = 1,
    Regular = 2,
    Extended = 3,
    Specific = 4,
}

impl TryFrom<u8> for ReadDeviceIdCode {
    type Error = u8;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        use ReadDeviceIdCode::*;
        match code {
            1 => Ok(Basic),
            2 => Ok(Regular),
            3 => Ok(Extended),
            4 => Ok(Specific),
            _ => Err(0),
        }
    }
}

impl From<ReadDeviceIdCode> for u8 {
    fn from(code: ReadDeviceIdCode) -> Self {
        use ReadDeviceIdCode::*;
        match code {
            Basic => 1,
            Regular => 2,
            Extended => 3,
            Specific => 4,
        }
    }
}
