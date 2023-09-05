use chrono::DateTime;
use std::io::Cursor;

use crate::ion_writer::IonWriter;
use crate::result::IonFailure;
use crate::serde::decimal::ION_DECIMAL;
use crate::serde::timestamp::ION_TIMESTAMP;
use crate::serde::SERDE_AS_ION;
use crate::types::Int;
use crate::{
    BinaryWriterBuilder, Element, IonError, IonResult, IonType, TextKind, TextWriterBuilder,
    Timestamp,
};
use serde::ser::Impossible;
use serde::{ser, Serialize};

/// Serialize an object into pretty formatted Ion text
pub fn to_pretty<T>(value: &T) -> IonResult<String>
where
    T: Serialize,
{
    SERDE_AS_ION.with(move |cell| {
        cell.set(true);
        let mut cursor = Cursor::new(Vec::new());
        let mut serializer = Serializer {
            writer: TextWriterBuilder::pretty().build(&mut cursor)?,
        };

        value.serialize(&mut serializer)?;
        serializer.writer.flush()?;
        drop(serializer);
        cell.set(false);

        let bytes = cursor.get_ref().clone();

        match String::from_utf8(bytes) {
            Ok(data) => Ok(data),
            Err(e) => IonResult::encoding_error(e.to_string()),
        }
    })
}

/// Serialize an object into compact Ion text format
pub fn to_string<T>(value: &T) -> IonResult<String>
where
    T: Serialize,
{
    SERDE_AS_ION.with(move |cell| {
        cell.set(true);
        let mut cursor = Cursor::new(Vec::new());
        let mut serializer = Serializer {
            writer: TextWriterBuilder::new(TextKind::Compact).build(&mut cursor)?,
        };

        value.serialize(&mut serializer)?;
        serializer.writer.flush()?;
        drop(serializer);
        cell.set(false);

        let bytes = cursor.get_ref().clone();

        match String::from_utf8(bytes) {
            Ok(data) => Ok(data),
            Err(e) => IonResult::encoding_error(e.to_string()),
        }
    })
}

/// Serialize an object into Ion binary format
pub fn to_binary<T>(value: &T) -> IonResult<Vec<u8>>
where
    T: Serialize,
{
    SERDE_AS_ION.with(move |cell| {
        cell.set(true);
        let mut cursor = Cursor::new(Vec::new());
        let mut serializer = Serializer {
            writer: BinaryWriterBuilder::new().build(&mut cursor)?,
        };

        value.serialize(&mut serializer)?;
        serializer.writer.flush()?;
        drop(serializer);
        cell.set(false);

        Ok(cursor.get_ref().clone())
    })
}

/// Implements a standard serializer for Ion
pub struct Serializer<E> {
    pub(crate) writer: E,
}

impl<'a, E> ser::Serializer for &'a mut Serializer<E>
where
    E: IonWriter,
{
    type Ok = ();
    type Error = IonError;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    /// Serialize a boolean to a bool value
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.writer.write_bool(v)
    }

    /// Serialize all integer types using the `Integer` intermediary type.
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.writer.write_int(&Int::from(v))
    }

    /// Serialize all integer types using the `Integer` intermediary type.
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.writer.write_int(&Int::from(v))
    }

    /// Serialize all integer types using the `Integer` intermediary type.
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.writer.write_int(&Int::from(v))
    }

    /// Serialize all integer types using the `Integer` intermediary type.
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.writer.write_int(&Int::from(v))
    }

    /// Serialize all integer types using the `Integer` intermediary type.
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.writer.write_int(&Int::from(v))
    }

    /// Serialize all integer types using the `Integer` intermediary type.
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.writer.write_int(&Int::from(v))
    }

    /// Serialize all integer types using the `Integer` intermediary type.
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.writer.write_int(&Int::from(v))
    }

    /// Serialize all integer types using the `Integer` intermediary type.
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.writer.write_int(&Int::from(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.writer.write_f32(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.writer.write_f64(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.writer.write_string(v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.writer.write_string(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.writer.write_blob(v)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_null(IonType::Null)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        if name == ION_TIMESTAMP {
            value.serialize(TimestampSerializer { serializer: self })
        } else if name == ION_DECIMAL {
            value.serialize(DecimalSerializer { serializer: self })
        } else {
            value.serialize(self)
        }
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.writer.step_in(IonType::Struct)?;
        self.writer.set_field_name(variant);
        value.serialize(&mut *self)?;
        self.writer.step_out()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.writer.step_in(IonType::List)?;
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.writer.step_in(IonType::List)?;
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.writer.step_in(IonType::List)?;
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.writer.step_in(IonType::List)?;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.writer.step_in(IonType::Struct)?;
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.writer.step_in(IonType::Struct)?;
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.writer.step_in(IonType::Struct)?;
        Ok(self)
    }
}

impl<'a, E> ser::SerializeSeq for &'a mut Serializer<E>
where
    E: IonWriter,
{
    type Ok = ();
    type Error = IonError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.step_out()
    }
}

impl<'a, E> ser::SerializeTuple for &'a mut Serializer<E>
where
    E: IonWriter,
{
    type Ok = ();
    type Error = IonError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.step_out()
    }
}

impl<'a, E> ser::SerializeTupleStruct for &'a mut Serializer<E>
where
    E: IonWriter,
{
    type Ok = ();
    type Error = IonError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.step_out()
    }
}

impl<'a, E> ser::SerializeTupleVariant for &'a mut Serializer<E>
where
    E: IonWriter,
{
    type Ok = ();
    type Error = IonError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.step_out()
    }
}

impl<'a, E> ser::SerializeMap for &'a mut Serializer<E>
where
    E: IonWriter,
{
    type Ok = ();
    type Error = IonError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        // We need to verify that the key is a string type or can be converted
        // to string
        let mk_serializer = MapKeySerializer {};
        let field: String = key.serialize(mk_serializer)?;
        self.writer.set_field_name(field);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.step_out()
    }
}

impl<'a, E> ser::SerializeStructVariant for &'a mut Serializer<E>
where
    E: IonWriter,
{
    type Ok = ();
    type Error = IonError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.writer.set_field_name(key);
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.step_out()
    }
}

impl<'a, E> ser::SerializeStruct for &'a mut Serializer<E>
where
    E: IonWriter,
{
    type Ok = ();
    type Error = IonError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.writer.set_field_name(key);
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), IonError> {
        self.writer.step_out()?;
        Ok(())
    }
}

/// This serializer is utilized for handling maps with ion. Ion
/// does not support non-string keys for maps. However, we can support
/// other key types as long as the key type implements to_string.
struct MapKeySerializer {}

fn key_must_be_a_string() -> IonError {
    IonError::encoding_error("Ion does not support non-string keys for maps".to_string())
}

impl ser::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = IonError;

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(variant.to_string())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    type SerializeSeq = Impossible<String, IonError>;
    type SerializeTuple = Impossible<String, IonError>;
    type SerializeTupleStruct = Impossible<String, IonError>;
    type SerializeTupleVariant = Impossible<String, IonError>;
    type SerializeMap = Impossible<String, IonError>;
    type SerializeStruct = Impossible<String, IonError>;
    type SerializeStructVariant = Impossible<String, IonError>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(key_must_be_a_string())
    }
}

/// Handles the `Timestamp` serialization by extracting the datetime
/// out of the interim structure and writing it properly to the Ion writer
pub struct TimestampSerializer<'a, E> {
    serializer: &'a mut Serializer<E>,
}

impl<'a, E> ser::Serializer for TimestampSerializer<'a, E>
where
    E: IonWriter,
{
    type Ok = ();
    type Error = IonError;

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let datetime =
            DateTime::parse_from_rfc3339(v).map_err(|e| IonError::encoding_error(e.to_string()))?;
        let timestamp = Timestamp::from(datetime);
        self.serializer.writer.write_timestamp(&timestamp)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        let datetime = DateTime::parse_from_rfc3339(variant)
            .map_err(|e| IonError::encoding_error(e.to_string()))?;
        let timestamp = Timestamp::from(datetime);
        self.serializer.writer.write_timestamp(&timestamp)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    type SerializeSeq = Impossible<(), IonError>;
    type SerializeTuple = Impossible<(), IonError>;
    type SerializeTupleStruct = Impossible<(), IonError>;
    type SerializeTupleVariant = Impossible<(), IonError>;
    type SerializeMap = Impossible<(), IonError>;
    type SerializeStruct = Impossible<(), IonError>;
    type SerializeStructVariant = Impossible<(), IonError>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(key_must_be_a_string())
    }
}

/// Serializer for Ion `Decimal`
pub struct DecimalSerializer<'a, E> {
    serializer: &'a mut Serializer<E>,
}

impl<'a, E> ser::Serializer for DecimalSerializer<'a, E>
where
    E: IonWriter,
{
    type Ok = ();
    type Error = IonError;

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let e = Element::read_one(v)?;
        let decimal = e.as_decimal().ok_or(IonError::encoding_error(format!(
            "Decimal serialization failed for: {}",
            v
        )))?;
        self.serializer.writer.write_decimal(decimal)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        let e = Element::read_one(variant)?;
        // TODO: remove unwrap
        let decimal = e.as_decimal().unwrap();
        self.serializer.writer.write_decimal(decimal)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    type SerializeSeq = Impossible<(), IonError>;
    type SerializeTuple = Impossible<(), IonError>;
    type SerializeTupleStruct = Impossible<(), IonError>;
    type SerializeTupleVariant = Impossible<(), IonError>;
    type SerializeMap = Impossible<(), IonError>;
    type SerializeStruct = Impossible<(), IonError>;
    type SerializeStructVariant = Impossible<(), IonError>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(key_must_be_a_string())
    }
}