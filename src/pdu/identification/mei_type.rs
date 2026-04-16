#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeiType {
    DeviceIdentification = 0x0E,
}

impl TryFrom<u8> for MeiType {
    type Error = u8;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        use MeiType::*;
        match code {
            0x0E => Ok(DeviceIdentification),
            _ => Err(0),
        }
    }
}

impl From<MeiType> for u8 {
    fn from(code: MeiType) -> Self {
        use MeiType::*;
        match code {
            DeviceIdentification => 0x0E,
        }
    }
}
