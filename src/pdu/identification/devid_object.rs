use core::iter::once;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// 247 bytes max
pub struct DeviceIdentificationObject<'a> {
    pub id: u8,
    pub length: u8,
    // 245 bytes max
    pub value: &'a [u8],
}

impl<'a> DeviceIdentificationObject<'a> {
    pub fn bytes_len(&self) -> usize {
        2 + self.value.len()
    }

    pub fn bytes(&self) -> impl Iterator<Item = u8> {
        once(self.id)
            .chain(once(self.length))
            .chain(self.value.iter().copied())
    }

    pub fn new_empty() -> Self {
        Self {
            id: 0,
            length: 0,
            value: &[0; 245],
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for DeviceIdentificationObject<'a> {
    type Error = u8;

    fn try_from(code: &'a [u8]) -> Result<Self, Self::Error> {
        let length = code[1];

        Ok(Self {
            id: code[0],
            length,
            value: &code[2..length as usize + 2],
        })
    }
}
