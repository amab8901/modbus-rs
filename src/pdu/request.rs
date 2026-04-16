use crate::{
    error::{DecodeError, EncodeError, ExceptionError},
    exception_code::ExceptionCode,
    pdu::identification::{ObjectId, mei_type::MeiType, read_device_id_code::ReadDeviceIdCode},
};

use super::{
    Address, DataCoils, DataWords, Quantity, coil_to_u16_coil, function_code::FunctionCode,
    u16_coil_to_coil,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Request<'a> {
    ReadDeviceIdentification(MeiType, ReadDeviceIdCode, ObjectId),
    ReadCoils(Address, Quantity),
    ReadDiscreteInput(Address, Quantity),
    ReadHoldingRegisters(Address, Quantity),
    ReadInputRegisters(Address, Quantity),
    WriteSingleCoil(Address, bool),
    WriteSingleRegister(Address, u16),
    WriteMultipleCoils(Address, DataCoils<'a>),
    WriteMultipleRegisters(Address, DataWords<'a>),
    MaskWriteRegister(Address, u16, u16),
    ReadWriteMultipleRegisters(Address, Quantity, Address, DataWords<'a>),
    Custom(FunctionCode, &'a [u8]),
}

impl<'a> Request<'a> {
    pub fn pdu_len(&self) -> usize {
        match &self {
            Request::ReadDeviceIdentification(_, _, _) => 4,
            Request::ReadCoils(_, _)
            | Request::ReadDiscreteInput(_, _)
            | Request::ReadHoldingRegisters(_, _)
            | Request::ReadInputRegisters(_, _)
            | Request::WriteSingleCoil(_, _)
            | Request::WriteSingleRegister(_, _) => 5,
            Request::WriteMultipleCoils(_, coils) => 6 + coils.data().len(),
            Request::WriteMultipleRegisters(_, words) => 6 + words.data().len(),
            Request::MaskWriteRegister(_, _, _) => 7,
            Request::ReadWriteMultipleRegisters(_, _, _, words) => 10 + words.data().len(),
            Request::Custom(_, d) => 1 + d.len(),
        }
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        if self.pdu_len() > buf.len() {
            return Err(EncodeError::InvalidBufferSize);
        }

        buf[0] = FunctionCode::from(self).into();

        match &self {
            Request::ReadDeviceIdentification(mei_type, read_device_id_code, object_id) => {
                buf[1] = *mei_type as u8;
                buf[2] = *read_device_id_code as u8;
                buf[3] = *object_id;
            }
            Request::ReadCoils(address, quantity)
            | Request::ReadDiscreteInput(address, quantity)
            | Request::ReadHoldingRegisters(address, quantity)
            | Request::ReadInputRegisters(address, quantity) => {
                buf[1..3].copy_from_slice(&address.to_be_bytes());
                buf[3..5].copy_from_slice(&quantity.to_be_bytes());
            }
            Request::WriteSingleCoil(address, coil) => {
                let data = coil_to_u16_coil(*coil);
                buf[1..3].copy_from_slice(&address.to_be_bytes());
                buf[3..5].copy_from_slice(&data.to_be_bytes());
            }
            Request::WriteSingleRegister(address, word) => {
                buf[1..3].copy_from_slice(&address.to_be_bytes());
                buf[3..5].copy_from_slice(&word.to_be_bytes());
            }
            Request::WriteMultipleCoils(address, coils) => {
                buf[1..3].copy_from_slice(&address.to_be_bytes());
                buf[3..5].copy_from_slice(&coils.quantity().to_be_bytes());
                buf[5] = coils.data().len() as u8;
                buf[6..coils.data().len() + 6].copy_from_slice(coils.data());
            }
            Request::WriteMultipleRegisters(address, words) => {
                buf[1..3].copy_from_slice(&address.to_be_bytes());
                buf[3..5].copy_from_slice(&words.quantity().to_be_bytes());
                buf[5] = words.data().len() as u8;
                buf[6..words.data().len() + 6].copy_from_slice(words.data());
            }
            Request::MaskWriteRegister(address, and_mask, or_mask) => {
                buf[1..3].copy_from_slice(&address.to_be_bytes());
                buf[3..5].copy_from_slice(&and_mask.to_be_bytes());
                buf[5..7].copy_from_slice(&or_mask.to_be_bytes());
            }
            Request::ReadWriteMultipleRegisters(
                read_address,
                read_quantity,
                write_address,
                write_words,
            ) => {
                buf[1..3].copy_from_slice(&read_address.to_be_bytes());
                buf[3..5].copy_from_slice(&read_quantity.to_be_bytes());
                buf[5..7].copy_from_slice(&write_address.to_be_bytes());
                buf[7..9].copy_from_slice(&write_words.quantity().to_be_bytes());
                buf[9..write_words.data().len() + 9].copy_from_slice(write_words.data());
            }
            Request::Custom(_, data) => {
                buf[1..1 + data.len()].copy_from_slice(data);
            }
        }

        Ok(self.pdu_len())
    }

    pub fn decode(buf: &'a [u8]) -> Result<Self, DecodeError> {
        if buf.is_empty() {
            return Err(DecodeError::IncompleteBuffer {
                current_size: 0,
                min_needed_size: 1,
            });
        }

        let fn_code: FunctionCode = buf[0].try_into().map_err(|c| {
            if buf.len() > 1 {
                DecodeError::ModbusExceptionCode(
                    FunctionCode::try_from(c & 0x7f).unwrap(),
                    ExceptionCode::try_from(buf[1]),
                )
            } else {
                DecodeError::IncompleteBuffer {
                    current_size: 1,
                    min_needed_size: 2,
                }
            }
        })?;

        let request = match fn_code {
            FunctionCode::ReadCoils
            | FunctionCode::ReadDiscreteInput
            | FunctionCode::ReadHoldingRegisters
            | FunctionCode::ReadInputRegisters => {
                if 5 > buf.len() {
                    return Err(DecodeError::IncompleteBuffer {
                        current_size: buf.len(),
                        min_needed_size: 5,
                    });
                }
                let address = u16::from_be_bytes(buf[1..3].try_into().unwrap());
                let quantity = u16::from_be_bytes(buf[3..5].try_into().unwrap());

                match fn_code {
                    FunctionCode::ReadCoils => {
                        if quantity == 0 || quantity > 0x07d0 {
                            return Err(DecodeError::ModbusExceptionError(
                                fn_code,
                                ExceptionError::IllegalDataValue,
                            ));
                        }
                        Request::ReadCoils(address, quantity)
                    }
                    FunctionCode::ReadDiscreteInput => {
                        if quantity == 0 || quantity > 0x07d0 {
                            return Err(DecodeError::ModbusExceptionError(
                                fn_code,
                                ExceptionError::IllegalDataValue,
                            ));
                        }
                        Request::ReadDiscreteInput(address, quantity)
                    }
                    FunctionCode::ReadHoldingRegisters => {
                        if quantity == 0 || quantity > 0x7d {
                            return Err(DecodeError::ModbusExceptionError(
                                fn_code,
                                ExceptionError::IllegalDataValue,
                            ));
                        }
                        Request::ReadHoldingRegisters(address, quantity)
                    }
                    FunctionCode::ReadInputRegisters => {
                        if quantity == 0 || quantity > 0x7d {
                            return Err(DecodeError::ModbusExceptionError(
                                fn_code,
                                ExceptionError::IllegalDataValue,
                            ));
                        }
                        Request::ReadInputRegisters(address, quantity)
                    }
                    _ => unreachable!(),
                }
            }
            FunctionCode::WriteSingleCoil | FunctionCode::WriteSingleRegister => {
                if 5 > buf.len() {
                    return Err(DecodeError::IncompleteBuffer {
                        current_size: buf.len(),
                        min_needed_size: 5,
                    });
                }
                let address = u16::from_be_bytes(buf[1..3].try_into().unwrap());
                let value = u16::from_be_bytes(buf[3..5].try_into().unwrap());

                match fn_code {
                    FunctionCode::WriteSingleCoil => {
                        let Some(coil_bool) = u16_coil_to_coil(value) else {
                            return Err(DecodeError::ModbusExceptionError(
                                fn_code,
                                ExceptionError::IllegalDataValue,
                            ));
                        };
                        Request::WriteSingleCoil(address, coil_bool)
                    }
                    FunctionCode::WriteSingleRegister => {
                        Request::WriteSingleRegister(address, value)
                    }
                    _ => unreachable!(),
                }
            }
            FunctionCode::WriteMultipleCoils => {
                if 6 > buf.len() {
                    return Err(DecodeError::IncompleteBuffer {
                        current_size: buf.len(),
                        min_needed_size: 6,
                    });
                }
                let address = u16::from_be_bytes(buf[1..3].try_into().unwrap());
                let quantity = u16::from_be_bytes(buf[3..5].try_into().unwrap());
                if quantity == 0 || quantity > 0x07b0 {
                    return Err(DecodeError::ModbusExceptionError(
                        fn_code,
                        ExceptionError::IllegalDataValue,
                    ));
                }
                let byte_count = buf[5] as usize;
                if byte_count + 6 > buf.len() {
                    return Err(DecodeError::IncompleteBuffer {
                        current_size: buf.len(),
                        min_needed_size: byte_count + 6,
                    });
                }
                let data = &buf[6..byte_count + 6];
                Request::WriteMultipleCoils(address, DataCoils::new(data, quantity as usize))
            }
            FunctionCode::WriteMultipleRegisters => {
                if 6 > buf.len() {
                    return Err(DecodeError::IncompleteBuffer {
                        current_size: buf.len(),
                        min_needed_size: 6,
                    });
                }
                let address = u16::from_be_bytes(buf[1..3].try_into().unwrap());
                let quantity = u16::from_be_bytes(buf[3..5].try_into().unwrap());
                if quantity == 0 || quantity > 0x7b {
                    return Err(DecodeError::ModbusExceptionError(
                        fn_code,
                        ExceptionError::IllegalDataValue,
                    ));
                }
                let byte_count = buf[5] as usize;
                if byte_count + 6 > buf.len() {
                    return Err(DecodeError::IncompleteBuffer {
                        current_size: buf.len(),
                        min_needed_size: byte_count + 6,
                    });
                }
                let data = &buf[6..byte_count + 6];
                Request::WriteMultipleRegisters(address, DataWords::new(data, quantity as usize))
            }
            FunctionCode::ReadDeviceIdentification => {
                if 5 < buf.len() {
                    return Err(DecodeError::IncompleteBuffer {
                        current_size: buf.len(),
                        min_needed_size: 4,
                    });
                }
                let mei_type = MeiType::try_from(buf[1]).unwrap();
                let read_device_id_code = ReadDeviceIdCode::try_from(buf[2]).unwrap();
                let object_id = buf[3];

                Request::ReadDeviceIdentification(mei_type, read_device_id_code, object_id)
            }
            FunctionCode::MaskWriteRegister => {
                if 7 > buf.len() {
                    return Err(DecodeError::IncompleteBuffer {
                        current_size: buf.len(),
                        min_needed_size: 7,
                    });
                }
                let reference_address = u16::from_be_bytes(buf[1..3].try_into().unwrap());
                let and_mask = u16::from_be_bytes(buf[3..5].try_into().unwrap());
                let or_mask = u16::from_be_bytes(buf[5..7].try_into().unwrap());
                Request::MaskWriteRegister(reference_address, and_mask, or_mask)
            }
            FunctionCode::ReadWriteMultipleRegisters => {
                if 10 > buf.len() {
                    return Err(DecodeError::IncompleteBuffer {
                        current_size: buf.len(),
                        min_needed_size: 10,
                    });
                }
                let read_address = u16::from_be_bytes(buf[1..3].try_into().unwrap());
                let read_quantity = u16::from_be_bytes(buf[3..5].try_into().unwrap());
                if read_quantity == 0 || read_quantity > 0x7d {
                    return Err(DecodeError::ModbusExceptionError(
                        fn_code,
                        ExceptionError::IllegalDataValue,
                    ));
                }
                let write_address = u16::from_be_bytes(buf[5..7].try_into().unwrap());
                let write_quantity = u16::from_be_bytes(buf[7..9].try_into().unwrap());
                if write_quantity == 0 || write_quantity > 0x7d {
                    return Err(DecodeError::ModbusExceptionError(
                        fn_code,
                        ExceptionError::IllegalDataValue,
                    ));
                }
                let write_byte_count = buf[9] as usize;
                if write_byte_count + 10 > buf.len() {
                    return Err(DecodeError::IncompleteBuffer {
                        current_size: buf.len(),
                        min_needed_size: write_byte_count + 10,
                    });
                }
                let data = &buf[10..write_byte_count + 10];
                Request::ReadWriteMultipleRegisters(
                    read_address,
                    read_quantity,
                    write_address,
                    DataWords::new(data, write_quantity as usize),
                )
            }
            FunctionCode::Custom(_) => Request::Custom(fn_code, &buf[1..]),
        };

        Ok(request)
    }
}

impl<'a> TryFrom<&'a [u8]> for Request<'a> {
    type Error = DecodeError;

    fn try_from(buf: &'a [u8]) -> Result<Self, Self::Error> {
        Self::decode(buf)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        error::ExceptionError,
        exception_code::ExceptionCode,
        pdu::{DataCoils, function_code::FunctionCode},
    };

    use super::{DecodeError, Request};

    #[test]
    fn request_from_buffer() {
        let buf: &[u8] = &[];
        assert_eq!(
            Request::try_from(buf),
            Err(DecodeError::IncompleteBuffer {
                current_size: 0,
                min_needed_size: 1,
            })
        );

        let buf: &[u8] = &[0x81, 0x01];
        assert_eq!(
            Request::try_from(buf),
            Err(DecodeError::ModbusExceptionCode(
                FunctionCode::ReadCoils,
                Ok(ExceptionCode::IllegalFunction)
            ))
        );
        let buf: &[u8] = &[0x90, 0x02];
        assert_eq!(
            Request::try_from(buf),
            Err(DecodeError::ModbusExceptionCode(
                FunctionCode::WriteMultipleRegisters,
                Ok(ExceptionCode::IllegalDataAddress)
            ))
        );

        let buf: &[u8] = &[0x01, 0x00, 0x06, 0x03, 0xe8];
        assert_eq!(Request::try_from(buf), Ok(Request::ReadCoils(6, 1000)));
        let buf: &[u8] = &[0x01, 0x00, 0x06, 0x80, 0x00];
        assert_eq!(
            Request::try_from(buf),
            Err(DecodeError::ModbusExceptionError(
                FunctionCode::ReadCoils,
                ExceptionError::IllegalDataValue,
            ))
        );

        let buf: &[u8] = &[0x0f, 0x00, 0x13, 0x00, 0x0a, 0x02, 0xcd, 0x01];
        assert_eq!(
            Request::try_from(buf),
            Ok(Request::WriteMultipleCoils(
                0x13,
                DataCoils::new(&[0xcd, 0x01], 0x0a)
            ))
        );
    }

    #[test]
    fn buffer_from_response() {
        let res = Request::ReadCoils(0x03e8, 0x0123);
        let buf: &mut [u8] = &mut [0; 5];
        let pdu_len = res.encode(buf);
        assert_eq!(pdu_len, Ok(5));
        assert_eq!(buf, &[0x01, 0x03, 0xe8, 0x01, 0x23]);
    }
}
