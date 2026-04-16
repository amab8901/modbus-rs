#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConformityLevel {
    BasicStreamOnly = 0x01,
    RegularStreamOnly = 0x02,
    ExtendedStreamOnly = 0x03,
    BasicStreamAndIndividual = 0x81,
    RegularStreamAndIndividual = 0x82,
    ExtendedStreamAndIndividual = 0x83,
}

impl TryFrom<u8> for ConformityLevel {
    type Error = u8;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        use ConformityLevel::*;
        match code {
            0x01 => Ok(BasicStreamOnly),
            0x02 => Ok(RegularStreamOnly),
            0x03 => Ok(ExtendedStreamOnly),
            0x81 => Ok(BasicStreamAndIndividual),
            0x82 => Ok(RegularStreamAndIndividual),
            0x83 => Ok(ExtendedStreamAndIndividual),
            _ => Err(0),
        }
    }
}

impl From<ConformityLevel> for u8 {
    fn from(code: ConformityLevel) -> Self {
        use ConformityLevel::*;
        match code {
            BasicStreamOnly => 0x01,
            RegularStreamOnly => 0x02,
            ExtendedStreamOnly => 0x03,
            BasicStreamAndIndividual => 0x81,
            RegularStreamAndIndividual => 0x82,
            ExtendedStreamAndIndividual => 0x83,
        }
    }
}
