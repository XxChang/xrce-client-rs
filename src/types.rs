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
pub struct CLIENT_Representation {
    pub xrce_cookie: XrceCookie,
    pub xrce_version: XrceVersion,
    pub xrce_vendor_id: XrceVendorId,
    pub client_key: ClientKey,
    pub session_id: u8,
    pub properties: Option<PropertySeq>,
    pub mtu: u16,
}

#[derive(Debug)]
pub struct Property {
    pub name: &'static str,
    pub value: &'static str,
}

type PropertySeq = [Property;UXR_PROPERTY_SEQUENCE_MAX] ;

#[allow(non_camel_case_types)]
pub struct CREATE_CLIENT_Payload {
    pub client_representation: CLIENT_Representation,
}

impl Serialize for CREATE_CLIENT_Payload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        
        let mut s = serializer.serialize_tuple(0)?;
        
        s.serialize_element(&self.client_representation.xrce_cookie[0])?;
        s.serialize_element(&self.client_representation.xrce_cookie[1])?;
        s.serialize_element(&self.client_representation.xrce_cookie[2])?;
        s.serialize_element(&self.client_representation.xrce_cookie[3])?;
        
        s.serialize_element(&self.client_representation.xrce_version[0])?;
        s.serialize_element(&self.client_representation.xrce_version[1])?;

        s.serialize_element(&self.client_representation.xrce_vendor_id[0])?;
        s.serialize_element(&self.client_representation.xrce_vendor_id[1])?;

        s.serialize_element(&self.client_representation.client_key[0])?;
        s.serialize_element(&self.client_representation.client_key[1])?;
        s.serialize_element(&self.client_representation.client_key[2])?;
        s.serialize_element(&self.client_representation.client_key[3])?;

        s.serialize_element(&self.client_representation.session_id)?;

        if let Some(property_seq) = &self.client_representation.properties {
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

        s.serialize_element(&self.client_representation.mtu)?;

        s.end()
    }
}

const CREATE_CLIENT_PAYLOAD_SIZE:usize = 16;

impl CREATE_CLIENT_Payload {
    pub fn to_slice(self, buf: &mut [u8]) -> error::Result<()> {
        let mut ucdr = micro_cdr::Encoder::new(buf);

        // TODO add properties
        SubMessageHeader::CreateClient(CREATE_CLIENT_PAYLOAD_SIZE as u16).serialize(&mut ucdr)?;
        self.serialize(&mut ucdr)?;

        Ok(())
    }
}
