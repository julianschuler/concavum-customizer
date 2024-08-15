use std::{fmt::Display, result};

use itoa::{Buffer, Integer};
use serde::{
    ser::{
        Error as SerdeError, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
        SerializeTuple, SerializeTupleStruct, SerializeTupleVariant, Serializer as SerdeSerializer,
    },
    Serialize,
};

use crate::unit::MM_TO_UNIT;

type Result<T> = result::Result<T, Error>;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    /// A custom error message.
    #[error("{0}")]
    Message(String),

    /// The top-level element can not be a sequence.
    #[error("the top-level element can not be a sequence")]
    TopLevelSequence,
}

impl SerdeError for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Message(msg.to_string())
    }
}

pub struct Serializer {
    output: String,
    indentation_level: usize,
    itoa_buffer: Buffer,
    sequence_element_names: Vec<String>,
}

impl Serializer {
    /// Creates a new serializer with the given top-level name.
    pub fn new(name: &str) -> Self {
        let mut serializer = Self {
            output: String::new(),
            indentation_level: 0,
            itoa_buffer: Buffer::new(),
            sequence_element_names: Vec::new(),
        };

        serializer.begin_s_expression();
        serializer.output += name;

        serializer
    }

    /// Finishes the serialization and returns the output.
    pub fn finish(mut self) -> String {
        self.end_s_expression();
        self.output += "\n";
        self.output
    }

    fn newline(&mut self) {
        self.output += "\n";
        for _ in 0..self.indentation_level {
            self.output += "\t";
        }
    }

    fn begin_s_expression(&mut self) {
        if self.indentation_level > 0 {
            self.newline();
        }
        self.output += "(";
        self.indentation_level += 1;
    }

    fn end_s_expression(&mut self) {
        self.indentation_level -= 1;
        if self.output.ends_with(')') {
            self.newline();
        }
        self.output += ")";
    }

    fn space_if_needed(&mut self) {
        if !self.output.ends_with('(') {
            self.output += " ";
        }
    }

    fn serialize_integer(&mut self, integer: impl Integer) {
        self.space_if_needed();
        self.output += self.itoa_buffer.format(integer);
    }

    fn start_remove_level(&mut self) -> Result<()> {
        // Remove the last level
        let position = self.output.rfind('\n').ok_or(Error::TopLevelSequence)?;
        let mut removed = self.output.split_off(position);
        self.indentation_level -= 1;

        // Extract the (plural) sequence name from the removed string
        let position = removed.rfind('(').ok_or(Error::TopLevelSequence)?;
        let mut sequence_name = removed.split_off(position + 1);

        // Remove the last character to get singular element name
        sequence_name.pop();
        self.sequence_element_names.push(sequence_name);

        Ok(())
    }

    fn end_remove_level(&mut self) {
        self.sequence_element_names.pop();
        self.output.pop();
        self.indentation_level += 1;
    }

    fn add_sequence_element_name(&mut self) {
        if let Some(name) = self.sequence_element_names.last() {
            self.output += name;
        }
    }
}

impl<'a> SerdeSerializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, value: bool) -> Result<()> {
        self.space_if_needed();
        self.output += if value { "yes" } else { "no" };
        Ok(())
    }

    fn serialize_i8(self, value: i8) -> Result<()> {
        self.serialize_integer(value);
        Ok(())
    }

    fn serialize_i16(self, value: i16) -> Result<()> {
        self.serialize_integer(value);
        Ok(())
    }

    fn serialize_i32(self, value: i32) -> Result<()> {
        // KiCAD uses nanometers stored in an i32 as internal unit of length.
        // We assume here that all i32s are used to represent lengths.
        // For serialization, this value is converted to millimeters.
        let integer_part = value / MM_TO_UNIT;
        let fractional_part = value % MM_TO_UNIT;

        self.serialize_integer(integer_part);

        if fractional_part != 0 {
            let output = self
                .itoa_buffer
                .format(fractional_part)
                .trim_end_matches('0');
            self.output += ".";
            self.output += output;
        }

        Ok(())
    }

    fn serialize_i64(self, value: i64) -> Result<()> {
        self.serialize_integer(value);
        Ok(())
    }

    fn serialize_u8(self, value: u8) -> Result<()> {
        self.serialize_integer(value);
        Ok(())
    }

    fn serialize_u16(self, value: u16) -> Result<()> {
        self.serialize_integer(value);
        Ok(())
    }

    fn serialize_u32(self, value: u32) -> Result<()> {
        self.serialize_integer(value);
        Ok(())
    }

    fn serialize_u64(self, value: u64) -> Result<()> {
        // This is implemented to match KiCADs formatting of LSETs.
        let upper = (value >> 32) as u32;
        #[allow(clippy::cast_possible_truncation)]
        let lower = value as u32;

        self.space_if_needed();
        self.output += &format!("{upper:#09x}_{lower:08x}");

        Ok(())
    }

    fn serialize_f32(self, value: f32) -> Result<()> {
        self.serialize_f64(f64::from(value))
    }

    fn serialize_f64(self, value: f64) -> Result<()> {
        self.space_if_needed();
        self.output += &format!("{value:.6}");
        Ok(())
    }

    fn serialize_char(self, value: char) -> Result<()> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_str(self, value: &str) -> Result<()> {
        self.space_if_needed();
        self.output += "\"";
        self.output += &value.replace('\\', r"\\").replace('"', r#"\""#);
        self.output += "\"";
        Ok(())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        let mut sequence = self.serialize_seq(Some(value.len()))?;
        for byte in value {
            SerializeSeq::serialize_element(&mut sequence, byte)?;
        }
        SerializeSeq::end(sequence)
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.space_if_needed();
        self.output += &variant.to_lowercase();
        Ok(())
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.begin_s_expression();
        value.serialize(&mut *self)?;
        self.end_s_expression();

        Ok(())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_newtype_struct(variant, value)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.start_remove_level()?;

        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_tuple(len)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.serialize_struct(variant, len)
    }
}

impl<'a> SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.begin_s_expression();
        self.add_sequence_element_name();
        value.serialize(&mut **self)?;
        self.end_s_expression();

        Ok(())
    }

    fn end(self) -> Result<()> {
        self.end_remove_level();

        Ok(())
    }
}

impl<'a> SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.begin_s_expression();
        key.serialize(&mut **self)?;

        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        self.end_s_expression();

        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.begin_s_expression();
        self.output += key;
        value.serialize(&mut **self)?;
        self.end_s_expression();

        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.begin_s_expression();
        self.output += key;
        value.serialize(&mut **self)?;
        self.end_s_expression();

        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}
