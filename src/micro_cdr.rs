use core::ptr ;
use serde::Serializer;
use serde::de;
use serde::Deserializer;
use core::marker::PhantomData;
use crate::Endianness;

use crate::error::{Error, self} ;

#[cfg(feature = "little")]
pub const NATIVE_ENDIANNESS: Endianness = Endianness::LittleEndianness ;

#[cfg(feature = "big")]
pub const NATIVE_ENDIANNESS: Endianness = Endianness::BigEndianness ;

#[cfg(feature = "big")]
compile_error!("big endianness not supported yet!");

#[cfg(all(feature = "little", feature = "big"))]
compile_error!("feature \"little\" and feature \"big\" cannot be enabled at the same time");

pub struct Encoder<'storage> {
    // buf: &'storage mut [u8],
    pos: *mut u8,
    end: *mut u8,
    offset: usize,
    endianness: Endianness,
    _p: PhantomData<&'storage [u8]>
}

impl<'storage> Encoder<'storage> {
    pub fn new(buffer: &'storage mut [u8]) -> Self {
        let ptr = buffer.as_mut_ptr() ;
        Encoder { 
            // buf: buffer, 
            pos: ptr, 
            end: unsafe {
                ptr.add(buffer.len())
            },
            offset: 0,
            endianness: NATIVE_ENDIANNESS,
            _p: PhantomData,
        }
    }

    pub fn new_with_endianness(buffer: &'storage mut [u8], endianness: Endianness) -> Self {
        let ptr = buffer.as_mut_ptr() ;
        Encoder { 
            // buf: buffer, 
            pos: ptr, 
            end: unsafe {
                ptr.add(buffer.len())
            },
            offset: 0,
            endianness,
            _p: PhantomData,
        }
    }

    fn set_pos_of<T>(&mut self) -> error::Result<()> {
        let alignment = core::mem::size_of::<T>();
        let rem_mask = alignment - 1;
        
        match self.offset & rem_mask {
            0 => {  },
            n @ 1..=7 => {
                let amt = alignment - n ;
                self.pos = self.check_avaliable(amt)?;
                self.offset += amt ;
            },
            _ => unreachable!(),
        }

        Ok(())
    }

    fn check_avaliable(&mut self, bytes: usize) -> error::Result<*mut u8> {
        let new_pos = unsafe { self.pos.add(bytes) };
        if new_pos <= self.end {
            Ok(new_pos)
        } else {
            Err(error::Error::BufferNotEnough)
        }
    }

    fn write_usize_as_u32(&mut self, v: usize) -> error::Result<()> {
        if v > core::u32::MAX as usize {
            return Err(Error::NumberOutOfRange);
        }
        
        self.serialize_u32(v as u32)
    }
}

macro_rules! impl_serialize_value {
    ($ser_method:ident($ty:ty)) => {
        fn $ser_method(self, v: $ty) -> error::Result<Self::Ok> {
            self.set_pos_of::<$ty>()?;
            let len = core::mem::size_of::<$ty>();
            self.check_avaliable(len)?;
            unsafe {
                let data_ptr = ptr::addr_of!(v) as *const u8;
                if NATIVE_ENDIANNESS == self.endianness {
                    ptr::copy_nonoverlapping(data_ptr, self.pos, len) ;
                } else {
                    for i in 0..len {
                        *self.pos.add(i) = *data_ptr.add(len - 1 - i) ;
                    }
                }
                self.pos = self.pos.add(len);
            }
            self.offset += len;
            Ok(())
        }
    };
}

impl serde::Serializer for &mut Encoder<'_> {
    type Error = Error;
    type Ok = () ;
    
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeStructVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeTupleVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.set_pos_of::<bool>()?;
        self.check_avaliable(1)?;
        unsafe{ 
            ptr::copy_nonoverlapping(ptr::addr_of!(v) as *const u8, self.pos, 1) ;
            self.pos = self.pos.add(1);
        };
        self.offset += 1;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.set_pos_of::<i8>()?;
        self.check_avaliable(1)?;
        unsafe{ 
            ptr::copy_nonoverlapping(ptr::addr_of!(v) as *const u8, self.pos, 1);
            self.pos = self.pos.add(1);
        }
        self.offset += 1;
        Ok(())
    }

    impl_serialize_value! { serialize_i16(i16) }
    impl_serialize_value! { serialize_i32(i32) }
    impl_serialize_value! { serialize_i64(i64) }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.set_pos_of::<u8>()?;
        self.check_avaliable(1)?;
        unsafe{ 
            ptr::copy_nonoverlapping(ptr::addr_of!(v) as *const u8, self.pos, 1);
            self.pos = self.pos.add(1);
        } 
        self.offset += 1;
        Ok(())
    }

    impl_serialize_value! { serialize_u16(u16) }
    impl_serialize_value! { serialize_u32(u32) }
    impl_serialize_value! { serialize_u64(u64) }

    impl_serialize_value! { serialize_f32(f32) }
    impl_serialize_value! { serialize_f64(f64) }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        if !v.is_ascii() {
            Err(Error::InvalidChar(v))
        } else {
            let mut buf = [0u8; 1] ;
            v.encode_utf8(&mut buf);
            self.serialize_u8(buf[0])
        }
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        if !v.is_ascii() {
            Err(Error::InvalidString)
        } else {
            let l = v.len() + 1;
            self.write_usize_as_u32(l)?;
            self.check_avaliable(l)?;
            unsafe {
                let data_ptr = v.as_ptr();
                ptr::copy_nonoverlapping(data_ptr, self.pos, l);
                self.pos = self.pos.add(l);
            }
            self.offset += l ;
            Ok(())
        }
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let l = v.len() ;
        self.write_usize_as_u32(l)?;
        self.check_avaliable(l)?;
        unsafe {
            let data_ptr = v.as_ptr();
            ptr::copy_nonoverlapping(data_ptr, self.pos, l);
            self.pos = self.pos.add(l);
        }
        self.offset += l ;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_some<T: ?Sized>(self, _: &T) -> Result<Self::Ok, Self::Error>
        where
            T: serde::Serialize {
        unimplemented!()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
            self,
            _: &'static str,
            variant_index: u32,
            _: &'static str,
        ) -> Result<Self::Ok, Self::Error> {
        self.serialize_u32(variant_index)
    }

    fn serialize_newtype_struct<T: ?Sized>(
            self,
            _: &'static str,
            value: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: serde::Serialize {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
            self,
            _: &'static str,
            variant_index: u32,
            _: &'static str,
            value: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: serde::Serialize {
        self.serialize_u32(variant_index)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let len = len.ok_or(Error::SequenceMustHaveLength)?;
        self.write_usize_as_u32(len)?;
        Ok(self)
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
            self,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
            self,
            _: &'static str,
            variant_index: u32,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_u32(variant_index)?;
        Ok(self)
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }

    fn serialize_struct(
            self,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
            self,
            _: &'static str,
            variant_index: u32,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_u32(variant_index)?;
        Ok(self)
    }

    fn collect_str<T: ?Sized>(self, _: &T) -> Result<Self::Ok, Self::Error>
        where
            T: core::fmt::Display {
        unimplemented!()
    }
    
    fn is_human_readable(&self) -> bool {
        false
    }
}

impl serde::ser::SerializeSeq for &mut Encoder<'_> {
    type Ok = ();
    type Error = error::Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: serde::Serialize {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl serde::ser::SerializeTuple for &mut Encoder<'_>
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> error::Result<()>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> error::Result<()> {
        Ok(())
    }
}

impl serde::ser::SerializeTupleStruct for &mut Encoder<'_>
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> error::Result<()>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> error::Result<()> {
        Ok(())
    }
}

impl serde::ser::SerializeTupleVariant for &mut Encoder<'_>
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> error::Result<()>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> error::Result<()> {
        Ok(())
    }
}

impl serde::ser::SerializeMap for &mut Encoder<'_>
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> error::Result<()>
    where
        T: serde::ser::Serialize,
    {
        key.serialize(&mut **self)
    }

    #[inline]
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> error::Result<()>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> error::Result<()> {
        Ok(())
    }
}

impl serde::ser::SerializeStruct for &mut Encoder<'_>
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> error::Result<()>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> error::Result<()> {
        Ok(())
    }
}

impl serde::ser::SerializeStructVariant for &mut Encoder<'_>
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> error::Result<()>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> error::Result<()> {
        Ok(())
    }
}

pub struct Decoder<'storage> {
    pos: *const u8,
    end: *const u8,
    offset: usize,
    endianness: Endianness,
    _p: PhantomData<&'storage [u8]>
}

impl<'storage> Decoder<'storage> {
    pub fn new(buffer: &'storage mut [u8]) -> Self {
        let ptr = buffer.as_ptr() ;
        Decoder { 
            // buf: buffer, 
            pos: ptr, 
            end: unsafe {
                ptr.add(buffer.len())
            },
            offset: 0,
            endianness: NATIVE_ENDIANNESS,
            _p: PhantomData,
        }
    }

    pub fn new_with_endianness(buffer: &'storage mut [u8], endianness: Endianness) -> Self {
        let ptr = buffer.as_mut_ptr() ;
        Decoder { 
            // buf: buffer, 
            pos: ptr, 
            end: unsafe {
                ptr.add(buffer.len())
            },
            offset: 0,
            endianness,
            _p: PhantomData,
        }
    }

    fn set_pos_of<T>(&mut self) -> error::Result<()> {
        let alignment = core::mem::size_of::<T>();
        let rem_mask = alignment - 1;
        
        match self.offset & rem_mask {
            0 => {  },
            n @ 1..=7 => {
                let amt = alignment - n ;
                self.pos = self.check_avaliable(amt)?;
                self.offset += amt ;
            },
            _ => unreachable!(),
        }

        Ok(())
    }

    fn check_avaliable(&mut self, bytes: usize) -> error::Result<*const u8> {
        let new_pos = unsafe { self.pos.add(bytes) };
        if new_pos <= self.end {
            Ok(new_pos)
        } else {
            Err(error::Error::BufferNotEnough)
        }
    }

    fn read_str(&mut self) -> error::Result<&'storage str> {
        core::str::from_utf8(
            { 
                let v = self.read_bytes()?;
                let len = v.len();
                &v[..(len - 1)]
            }
        ).map_err(|e| Error::InvalidUtf8Encoding(e))
    }

    fn read_bytes(&mut self) -> error::Result<&'storage [u8]> {
       let len: u32 = de::Deserialize::deserialize(&mut *self)?;
       self.check_avaliable(len as usize)?;
       unsafe {
         let sli = core::slice::from_raw_parts(self.pos, len as usize);
         self.pos = self.pos.add(len as usize);
         Ok(sli)
       }
    }
}

macro_rules! impl_deserialize_value {
    ($de_method:ident<$ty:ty> = $visitor_method:ident ) => {
        fn $de_method<V>(self, visitor: V) -> error::Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            self.set_pos_of::<$ty>()?;
            let len = core::mem::size_of::<$ty>();
            self.check_avaliable(len)?;
            let v: $ty = <$ty>::default();
            unsafe {
                let data_ptr = ptr::addr_of!(v) as *mut u8;
                if NATIVE_ENDIANNESS == self.endianness {
                    ptr::copy_nonoverlapping(self.pos, data_ptr, len) ;
                } else {
                    for i in 0..len {
                        *data_ptr.add(i) = *self.pos.add(len - 1 - i) ;
                    }
                }
                self.pos = self.pos.add(len);
            }
            self.offset += len;
            visitor.$visitor_method(v)
        }
    };
}

impl<'de,> Deserializer<'de> for &mut Decoder<'de> 
{
    type Error = Error;
    
    fn deserialize_any<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de> {
        unimplemented!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de> {
        self.set_pos_of::<bool>()?;
        self.check_avaliable(1)?;
        let v = 0u8;
        unsafe{
            ptr::copy_nonoverlapping(self.pos, ptr::addr_of!(v) as *mut u8, 1);
            self.pos = self.pos.add(1);
        };
        self.offset += 1;
        match v {
            1 => visitor.visit_bool(true),
            0 => visitor.visit_bool(false),
            value => Err(Error::InvalidBoolEncoding(value)),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de> {
        self.set_pos_of::<i8>()?;
        self.check_avaliable(1)?;
        let v = 0i8;
        unsafe{
            ptr::copy_nonoverlapping(self.pos, ptr::addr_of!(v) as *mut u8, 1);
            self.pos = self.pos.add(1);
        };
        self.offset += 1;
        visitor.visit_i8(v)
    }

    impl_deserialize_value!(deserialize_i16<i16> = visit_i16 );
    impl_deserialize_value!(deserialize_i32<i32> = visit_i32 );
    impl_deserialize_value!(deserialize_i64<i64> = visit_i64 );

    fn deserialize_u8<V>(self, visitor: V) -> error::Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.set_pos_of::<u8>()?;
        self.check_avaliable(1)?;
        let v = 0u8;
        unsafe{
            ptr::copy_nonoverlapping(self.pos, ptr::addr_of!(v) as *mut u8, 1);
            self.pos = self.pos.add(1);
        };
        self.offset += 1;
        visitor.visit_u8(v)
    }

    impl_deserialize_value!(deserialize_u16<u16> = visit_u16);
    impl_deserialize_value!(deserialize_u32<u32> = visit_u32);
    impl_deserialize_value!(deserialize_u64<u64> = visit_u64);

    impl_deserialize_value!(deserialize_f32<f32> = visit_f32);
    impl_deserialize_value!(deserialize_f64<f64> = visit_f64);

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        self.set_pos_of::<u8>()?;
        self.check_avaliable(1)?;
        let v = [0u8;4];
        unsafe{
            ptr::copy_nonoverlapping(self.pos, ptr::addr_of!(v) as *mut u8, 1);

        };
        if utf8_char_width(v[0]) != 1 {
            Err(Error::InvalidCharEncoding)
        } else {
            unsafe {
                self.pos = self.pos.add(1);
            }
            self.offset += 1;
            visitor.visit_char(v[0] as char)
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {    
        visitor.visit_borrowed_str(&self.read_str()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        visitor.visit_borrowed_bytes(self.read_bytes()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        unimplemented!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
            self,
            _: &'static str,
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
            self,
            _: &'static str,
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        let len: u32 = de::Deserialize::deserialize(&mut *self)?;
        self.deserialize_tuple(len as usize, visitor)
    }

    fn deserialize_tuple<V>(self, _: usize, _: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        unimplemented!()
    }

    fn deserialize_tuple_struct<V>(
            self,
            _: &'static str,
            _: usize,
            _: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        unimplemented!()
    }

    fn deserialize_map<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        unimplemented!()
    }

    fn deserialize_struct<V>(
            self,
            _: &'static str,
            _: &'static [&'static str],
            _: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        unimplemented!()
    }

    fn deserialize_enum<V>(
            self,
            _: &'static str,
            _: &'static [&'static str],
            _: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        unimplemented!()
    }

    fn deserialize_ignored_any<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        unimplemented!()
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

#[inline]
fn utf8_char_width(first_byte: u8) -> usize {
    UTF8_CHAR_WIDTH[first_byte as usize] as usize
}

// https://tools.ietf.org/html/rfc3629
const UTF8_CHAR_WIDTH: &[u8; 256] = &[
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x1F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x3F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x5F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x7F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x9F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xBF
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, //
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 0xDF
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // 0xEF
    4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xFF
];
