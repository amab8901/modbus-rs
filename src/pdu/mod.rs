pub mod coil;
pub mod exception_response;
pub mod function_code;
pub mod identification;
pub mod request;
pub mod response;
pub mod word;

pub use coil::DataCoils;
pub use word::DataWords;

pub type Address = u16;
pub type Quantity = u16;

pub fn u16_coil_to_coil(u16_coil: u16) -> Option<bool> {
    match u16_coil {
        0x0000 => Some(false),
        0xff00 => Some(true),
        _ => None,
    }
}

pub fn coil_to_u16_coil(coil: bool) -> u16 {
    match coil {
        false => 0x0000,
        true => 0xff00,
    }
}
