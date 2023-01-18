use amf::{Amf0Value, Amf3Value};
use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "encoding: u8, payload_length: u32", id = "encoding")]
pub enum AmfValue {
    #[deku(id_pat = "18..=20")]
    Amf0(
        #[deku(
            reader = "read_amf0_value(deku::rest, payload_length)",
            writer = "write_amf_value(deku::output, &self)"
        )]
        Amf0Value,
    ),
    #[deku(id_pat = "15..=17")]
    Amf3(
        #[deku(
            reader = "read_amf3_value(deku::rest, payload_length)",
            writer = "write_amf_value(deku::output, &self)"
        )]
        Amf3Value,
    ),
}

impl From<Amf0Value> for AmfValue {
    fn from(val: Amf0Value) -> Self {
        Self::Amf0(val)
    }
}

impl From<Amf3Value> for AmfValue {
    fn from(val: Amf3Value) -> Self {
        Self::Amf3(val)
    }
}

fn read_amf0_value(
    input: &BitSlice<Msb0, u8>,
    payload_length: u32,
) -> Result<(&BitSlice<Msb0, u8>, Amf0Value), DekuError> {
    let (raw, rest) = input.split_at(payload_length as usize);
    match Amf0Value::read_from(raw.as_raw_slice()) {
        Ok(value) => Ok((rest, value)),
        Err(e) => Err(DekuError::Parse(e.to_string())),
    }
}

fn read_amf3_value(
    input: &BitSlice<Msb0, u8>,
    payload_length: u32,
) -> Result<(&BitSlice<Msb0, u8>, Amf3Value), DekuError> {
    let (raw, rest) = input.split_at(payload_length as usize);
    match Amf3Value::read_from(raw.as_raw_slice()) {
        Ok(value) => Ok((rest, value)),
        Err(e) => Err(DekuError::Parse(e.to_string())),
    }
}

fn write_amf_value(output: &mut BitVec<Msb0, u8>, value: &AmfValue) -> Result<(), DekuError> {
    if let Err(e) = match value {
        AmfValue::Amf0(amf0_val) => amf0_val.write_to(output),
        AmfValue::Amf3(amf3_val) => amf3_val.write_to(output),
    } {
        Err(DekuError::Unexpected(e.to_string()))
    } else {
        Ok(())
    }
}
