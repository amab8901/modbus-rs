#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoreFollows {
    NoMoreObjects = 0x00,
    OtherObjectsAvailable = 0xFF,
}

impl TryFrom<u8> for MoreFollows {
    type Error = u8;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        use MoreFollows::*;
        match code {
            0x00 => Ok(NoMoreObjects),
            0xFF => Ok(OtherObjectsAvailable),
            _ => Err(0),
        }
    }
}

impl From<MoreFollows> for u8 {
    fn from(code: MoreFollows) -> Self {
        use MoreFollows::*;
        match code {
            NoMoreObjects => 0x00,
            OtherObjectsAvailable => 0xFF,
        }
    }
}
