use serde::Deserialize;
use serde::de::Visitor;
use serde::Serialize;
use serde::ser::SerializeTuple;
use crate::error;
use crate::micro_cdr;
use crate::submessage::SubMessageHeader;

#[cfg(all(feature = "hard-liveliness-check", feature = "profile-shared-memory"))]
const UXR_PROPERTY_SEQUENCE_MAX:usize = 2;

#[cfg(all(feature = "hard-liveliness-check", not(feature = "profile-shared-memory")))]
const UXR_PROPERTY_SEQUENCE_MAX:usize = 1;

#[cfg(all(not(feature = "hard-liveliness-check"), feature = "profile-shared-memory"))]
const UXR_PROPERTY_SEQUENCE_MAX:usize = 1;

#[cfg(not(any(feature = "hard-liveliness-check", feature = "profile-shared-memory")))]
const UXR_PROPERTY_SEQUENCE_MAX:usize = 1;

type XrceCookie = [u8;4];
type XrceVersion = [u8;2];
type XrceVendorId = [u8;2];
type ClientKey = [u8;4];

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct CLIENT_Representation<'a> {
    pub xrce_cookie: XrceCookie,
    pub xrce_version: XrceVersion,
    pub xrce_vendor_id: XrceVendorId,
    pub client_key: ClientKey,
    pub session_id: u8,
    pub properties: Option<PropertySeq<'a>>,
    pub mtu: u16,
}

#[derive(Debug)]
pub struct Property<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

type PropertySeq<'a> = [Property<'a>;UXR_PROPERTY_SEQUENCE_MAX] ;

#[allow(non_camel_case_types)]
pub struct CREATE_CLIENT_Payload<'a> (pub CLIENT_Representation<'a>);

impl<'a> Serialize for CREATE_CLIENT_Payload<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        
        let mut s = serializer.serialize_tuple(0)?;
        
        s.serialize_element(&self.0.xrce_cookie[0])?;
        s.serialize_element(&self.0.xrce_cookie[1])?;
        s.serialize_element(&self.0.xrce_cookie[2])?;
        s.serialize_element(&self.0.xrce_cookie[3])?;
        
        s.serialize_element(&self.0.xrce_version[0])?;
        s.serialize_element(&self.0.xrce_version[1])?;

        s.serialize_element(&self.0.xrce_vendor_id[0])?;
        s.serialize_element(&self.0.xrce_vendor_id[1])?;

        s.serialize_element(&self.0.client_key[0])?;
        s.serialize_element(&self.0.client_key[1])?;
        s.serialize_element(&self.0.client_key[2])?;
        s.serialize_element(&self.0.client_key[3])?;

        s.serialize_element(&self.0.session_id)?;

        if let Some(property_seq) = &self.0.properties {
            let optional = true;
            s.serialize_element(&optional)?;
            for property in property_seq {
                s.serialize_element(property.name)?;
                s.serialize_element(property.value)?;
            }
        } else {
            let optional = false ;
            s.serialize_element(&optional)?;
        }

        s.serialize_element(&self.0.mtu)?;

        s.end()
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for CREATE_CLIENT_Payload<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        
        struct VisitorInside;
        
        impl<'de> Visitor<'de> for VisitorInside {
            type Value = CREATE_CLIENT_Payload<'de>;

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>, {
                use serde::de;
                Ok(CREATE_CLIENT_Payload(
                    CLIENT_Representation {    
                        xrce_cookie: [seq.next_element()?
                                        .ok_or_else(|| de::Error::invalid_length(0, &self))?,
                                      seq.next_element()?
                                        .ok_or_else(|| de::Error::invalid_length(1, &self))?,
                                      seq.next_element()?
                                        .ok_or_else(|| de::Error::invalid_length(2, &self))?,
                                      seq.next_element()?
                                        .ok_or_else(|| de::Error::invalid_length(3, &self))?],
                        xrce_version: [seq.next_element()?
                                        .ok_or_else(|| de::Error::invalid_length(4, &self))?,
                                       seq.next_element()?
                                        .ok_or_else(|| de::Error::invalid_length(5, &self))?],
                        xrce_vendor_id: [seq.next_element()?
                                            .ok_or_else(|| de::Error::invalid_length(6, &self))?,
                                         seq.next_element()?
                                            .ok_or_else(|| de::Error::invalid_length(7, &self))?],
                        client_key: [seq.next_element()?
                                        .ok_or_else(|| de::Error::invalid_length(8, &self))?,
                                     seq.next_element()?
                                        .ok_or_else(|| de::Error::invalid_length(9, &self))?,
                                     seq.next_element()?
                                        .ok_or_else(|| de::Error::invalid_length(10, &self))?,
                                     seq.next_element()?
                                        .ok_or_else(|| de::Error::invalid_length(11, &self))?],
                        session_id: seq.next_element()?
                                        .ok_or_else(|| de::Error::invalid_length(12, &self))?,
                        properties: if seq.next_element()?.ok_or_else(|| de::Error::invalid_length(13, &self))?
                                    {
                                        Some([Property {
                                            name: seq.next_element()?
                                        .ok_or_else(|| de::Error::invalid_length(3, &self))?,
                                            value: seq.next_element()?
                                        .ok_or_else(|| de::Error::invalid_length(3, &self))?,
                                        };UXR_PROPERTY_SEQUENCE_MAX])
                                    } else {
                                        None
                                    },
                        mtu: seq.next_element()?.ok_or_else(|| de::Error::invalid_length(3, &self))?,
                    }
                ))                
            }

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("struct CREATE_CLIENT_Payload")
            }
        }

        deserializer.deserialize_tuple_struct("", CREATE_CLIENT_PAYLOAD_SIZE + 20, VisitorInside)
    }
}

const CREATE_CLIENT_PAYLOAD_SIZE:usize = 16;

impl<'a> CREATE_CLIENT_Payload<'a> {
    pub fn to_slice(self, buf: &mut [u8]) -> error::Result<usize> {
        let mut ucdr = micro_cdr::Encoder::new(buf);

        // TODO add properties
        SubMessageHeader::CreateClient(CREATE_CLIENT_PAYLOAD_SIZE as u16).serialize(&mut ucdr)?;
        self.serialize(&mut ucdr)?;

        Ok(ucdr.finalize())
    }

    #[cfg(test)]
    pub fn from_slice(buf: &[u8]) -> crate::error::Result<CREATE_CLIENT_Payload> {
        let mut ucdr = micro_cdr::Decoder::new(buf);
        CREATE_CLIENT_Payload::deserialize(&mut ucdr)
    }
}
