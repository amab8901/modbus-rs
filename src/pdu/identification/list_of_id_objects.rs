use crate::pdu::identification::devid_object::DeviceIdentificationObject;

#[derive(Debug, PartialEq, Eq)]
pub struct ListOfIdObjects<'a> {
    pub list_of_id_objects: [DeviceIdentificationObject<'a>; 256],
}

impl<'a> ListOfIdObjects<'a> {
    pub fn bytes_len(&self) -> usize {
        self.list_of_id_objects
            .iter()
            .map(DeviceIdentificationObject::bytes_len)
            .sum::<usize>()
    }

    pub fn bytes(&self) -> impl Iterator<Item = u8> {
        self.list_of_id_objects.iter().flat_map(|dio| dio.bytes())
    }
}
