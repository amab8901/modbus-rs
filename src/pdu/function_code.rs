use super::{request::Request as PduRequest, response::Response as PduResponse};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FunctionCode {
    ReadCoils,
    ReadDiscreteInput,
    ReadHoldingRegisters,
    ReadInputRegisters,
    WriteSingleCoil,
    WriteSingleRegister,
    WriteMultipleCoils,
    WriteMultipleRegisters,
    ReadDeviceIdentification,
    MaskWriteRegister,
    ReadWriteMultipleRegisters,
    Custom(u8),
}

impl TryFrom<u8> for FunctionCode {
    type Error = u8;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        use FunctionCode::*;
        match code {
            0x01 => Ok(ReadCoils),
            0x02 => Ok(ReadDiscreteInput),
            0x03 => Ok(ReadHoldingRegisters),
            0x04 => Ok(ReadInputRegisters),
            0x05 => Ok(WriteSingleCoil),
            0x06 => Ok(WriteSingleRegister),
            0x0F => Ok(WriteMultipleCoils),
            0x10 => Ok(WriteMultipleRegisters),
            0x2B => Ok(ReadDeviceIdentification),
            0x16 => Ok(MaskWriteRegister),
            0x17 => Ok(ReadWriteMultipleRegisters),
            0x80.. => Err(code),
            code => Ok(Custom(code)),
        }
    }
}

impl From<FunctionCode> for u8 {
    fn from(code: FunctionCode) -> Self {
        use FunctionCode::*;
        match code {
            ReadCoils => 0x01,
            ReadDiscreteInput => 0x02,
            ReadHoldingRegisters => 0x03,
            ReadInputRegisters => 0x04,
            WriteSingleCoil => 0x05,
            WriteSingleRegister => 0x06,
            WriteMultipleCoils => 0x0F,
            WriteMultipleRegisters => 0x10,
            ReadDeviceIdentification => 0x2B,
            MaskWriteRegister => 0x16,
            ReadWriteMultipleRegisters => 0x17,
            Custom(code) => code,
        }
    }
}

impl<'a> From<&PduResponse<'a>> for FunctionCode {
    fn from(value: &PduResponse<'a>) -> Self {
        match value {
            PduResponse::ReadCoils(_) => FunctionCode::ReadCoils,
            PduResponse::ReadDiscreteInput(_) => FunctionCode::ReadDiscreteInput,
            PduResponse::ReadHoldingRegisters(_) => FunctionCode::ReadHoldingRegisters,
            PduResponse::ReadInputRegisters(_) => FunctionCode::ReadInputRegisters,
            PduResponse::WriteSingleCoil(_, _) => FunctionCode::WriteSingleCoil,
            PduResponse::WriteSingleRegister(_, _) => FunctionCode::WriteSingleRegister,
            PduResponse::WriteMultipleCoils(_, _) => FunctionCode::WriteMultipleCoils,
            PduResponse::WriteMultipleRegisters(_, _) => FunctionCode::WriteMultipleRegisters,
            PduResponse::ReadDeviceIdentification(_, _, _, _, _, _, _, _) => {
                FunctionCode::ReadDeviceIdentification
            }
            PduResponse::MaskWriteRegister(_, _, _) => FunctionCode::MaskWriteRegister,
            PduResponse::ReadWriteMultipleRegisters(_) => FunctionCode::ReadWriteMultipleRegisters,
            PduResponse::Custom(fn_code, _) => *fn_code,
        }
    }
}

impl<'a> From<&PduRequest<'a>> for FunctionCode {
    fn from(value: &PduRequest<'a>) -> Self {
        match value {
            PduRequest::ReadCoils(_, _) => FunctionCode::ReadCoils,
            PduRequest::ReadDiscreteInput(_, _) => FunctionCode::ReadDiscreteInput,
            PduRequest::ReadHoldingRegisters(_, _) => FunctionCode::ReadHoldingRegisters,
            PduRequest::ReadInputRegisters(_, _) => FunctionCode::ReadInputRegisters,
            PduRequest::WriteSingleCoil(_, _) => FunctionCode::WriteSingleCoil,
            PduRequest::WriteSingleRegister(_, _) => FunctionCode::WriteSingleRegister,
            PduRequest::WriteMultipleCoils(_, _) => FunctionCode::WriteMultipleCoils,
            PduRequest::WriteMultipleRegisters(_, _) => FunctionCode::WriteMultipleRegisters,
            PduRequest::ReadDeviceIdentification(_, _, _) => FunctionCode::ReadDeviceIdentification,
            PduRequest::MaskWriteRegister(_, _, _) => FunctionCode::MaskWriteRegister,
            PduRequest::ReadWriteMultipleRegisters(_, _, _, _) => {
                FunctionCode::ReadWriteMultipleRegisters
            }
            PduRequest::Custom(fn_code, _) => *fn_code,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn function_code_from_u8() {
        assert_eq!(FunctionCode::try_from(0x01), Ok(FunctionCode::ReadCoils));
        assert_eq!(
            FunctionCode::try_from(0x05),
            Ok(FunctionCode::WriteSingleCoil)
        );
        assert_eq!(FunctionCode::try_from(0x20), Ok(FunctionCode::Custom(0x20)));
        assert_eq!(FunctionCode::try_from(0x80), Err(0x80));
        assert_eq!(FunctionCode::try_from(0x9a), Err(0x9a));
        assert_eq!(FunctionCode::try_from(0xff), Err(0xff));
    }
}
