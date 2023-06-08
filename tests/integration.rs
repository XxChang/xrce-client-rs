#![no_std]
#![no_main]

use defmt_rtt as _;
use stm32f1xx_hal as _;
use panic_probe as _;

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[cfg(test)]
#[defmt_test::tests]
mod tests {

    #[test]
    fn serialize_octet() {
        let mut buf = [0u8;256];
        let v = 32u8;
        let mut writer = xrce_client_rs::micro_cdr::Encoder::new(&mut buf);
        serde::Serializer::serialize_u8(&mut writer, v).unwrap() ;
        assert_eq!([0x20], buf[0..1]);

        let mut writer = xrce_client_rs::micro_cdr::Encoder::new_with_endianness(&mut buf, xrce_client_rs::Endianness::BigEndianness);
        serde::Serializer::serialize_u8(&mut writer, v).unwrap() ;
        assert_eq!([0x20], buf[0..1]);
    }

    #[test]
    fn serialize_char() {
        let mut buf = [0u8;256];
        let v = 'Z';
        let mut writer = xrce_client_rs::micro_cdr::Encoder::new(&mut buf);
        serde::Serializer::serialize_char(&mut writer, v).unwrap() ;
        assert_eq!([0x5a], buf[0..1]);

        let mut writer = xrce_client_rs::micro_cdr::Encoder::new_with_endianness(&mut buf, xrce_client_rs::Endianness::BigEndianness);
        serde::Serializer::serialize_char(&mut writer, v).unwrap() ;
        assert_eq!([0x5a], buf[0..1]);
    }

    #[test]
    fn serialize_wchar() {
        let mut buf = [0u8;256];
        let v = 'Å';
        let mut writer = xrce_client_rs::micro_cdr::Encoder::new(&mut buf);
        assert!(serde::Serializer::serialize_char(&mut writer, v).is_err()) ;

        let mut writer = xrce_client_rs::micro_cdr::Encoder::new_with_endianness(&mut buf, xrce_client_rs::Endianness::BigEndianness);
        assert!(serde::Serializer::serialize_char(&mut writer, v).is_err()) ;
    }

    #[test]
    fn serialize_ushort() {
        let mut buf = [0u8;256];
        let v = 65500u16;
        let mut writer = xrce_client_rs::micro_cdr::Encoder::new(&mut buf);
        serde::Serializer::serialize_u16(&mut writer, v).unwrap() ;
        assert_eq!([0xdc, 0xff], buf[0..2]);

        let mut writer = xrce_client_rs::micro_cdr::Encoder::new_with_endianness(&mut buf, xrce_client_rs::Endianness::BigEndianness);
        serde::Serializer::serialize_u16(&mut writer, v).unwrap() ;
        assert_eq!([0xff, 0xdc], buf[0..2]);
    }

    #[test]
    fn serialize_short() {
        let v = -32700i16;
        let mut buf = [0u8;256];
        let mut writer = xrce_client_rs::micro_cdr::Encoder::new(&mut buf);
        serde::Serializer::serialize_i16(&mut writer, v).unwrap() ;
        assert_eq!([0x44, 0x80], buf[0..2]);

        let mut writer = xrce_client_rs::micro_cdr::Encoder::new_with_endianness(&mut buf, xrce_client_rs::Endianness::BigEndianness);
        serde::Serializer::serialize_i16(&mut writer, v).unwrap() ;
        assert_eq!([0x80, 0x44], buf[0..2]);
    }

    #[test]
    fn serialize_ulong() {
        let v = 4294967200u32;
        let mut buf = [0u8;256];
        let mut writer = xrce_client_rs::micro_cdr::Encoder::new(&mut buf);
        serde::Serializer::serialize_u32(&mut writer, v).unwrap() ;
        assert_eq!([0xa0, 0xff, 0xff, 0xff], buf[0..4]);

        let mut writer = xrce_client_rs::micro_cdr::Encoder::new_with_endianness(&mut buf, xrce_client_rs::Endianness::BigEndianness);
        serde::Serializer::serialize_u32(&mut writer, v).unwrap() ;
        assert_eq!([0xff, 0xff, 0xff, 0xa0], buf[0..4]);
    }

    #[test]
    fn serialize_long() {
        let v = -2147483600i32;
        let mut buf = [0u8;256];
        let mut writer = xrce_client_rs::micro_cdr::Encoder::new(&mut buf);
        serde::Serializer::serialize_i32(&mut writer, v).unwrap() ;
        assert_eq!([0x30, 0x00, 0x00, 0x80], buf[0..4]);

        let mut writer = xrce_client_rs::micro_cdr::Encoder::new_with_endianness(&mut buf, xrce_client_rs::Endianness::BigEndianness);
        serde::Serializer::serialize_i32(&mut writer, v).unwrap() ;
        assert_eq!([0x80, 0x00, 0x00, 0x30], buf[0..4]);
    }

    #[test]
    fn serialize_longlong() {
        let v = -9223372036800i64;
        let mut buf = [0u8;256];
        let mut writer = xrce_client_rs::micro_cdr::Encoder::new(&mut buf);
        serde::Serializer::serialize_i64(&mut writer, v).unwrap() ;
        assert_eq!([0x40, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff], buf[0..8]);

        let mut writer = xrce_client_rs::micro_cdr::Encoder::new_with_endianness(&mut buf, xrce_client_rs::Endianness::BigEndianness);
        serde::Serializer::serialize_i64(&mut writer, v).unwrap() ;
        assert_eq!([0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x40], buf[0..8]);
    }

    #[test]
    fn serialize_float() {
        let v = core::f32::MIN_POSITIVE;
        let mut buf = [0u8;256];
        let mut writer = xrce_client_rs::micro_cdr::Encoder::new(&mut buf);
        serde::Serializer::serialize_f32(&mut writer, v).unwrap() ;
        assert_eq!([0x00, 0x00, 0x80, 0x00], buf[0..4]);

        let mut writer = xrce_client_rs::micro_cdr::Encoder::new_with_endianness(&mut buf, xrce_client_rs::Endianness::BigEndianness);
        serde::Serializer::serialize_f32(&mut writer, v).unwrap() ;
        assert_eq!([0x00, 0x80, 0x00, 0x00], buf[0..4]);
    }

    #[test]
    fn serialize_double() {
        let v = core::f64::MIN_POSITIVE;
        let mut buf = [0u8;256];
        let mut writer = xrce_client_rs::micro_cdr::Encoder::new(&mut buf);
        serde::Serializer::serialize_f64(&mut writer, v).unwrap() ;
        assert_eq!([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00], buf[0..8]);

        let mut writer = xrce_client_rs::micro_cdr::Encoder::new_with_endianness(&mut buf, xrce_client_rs::Endianness::BigEndianness);
        serde::Serializer::serialize_f64(&mut writer, v).unwrap() ;
        assert_eq!([0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], buf[0..8]);
    }

    #[test]
    fn serialize_bool() {
        let v = true;
        let mut buf = [0u8;256];
        let mut writer = xrce_client_rs::micro_cdr::Encoder::new(&mut buf);
        serde::Serializer::serialize_bool(&mut writer, v).unwrap() ;
        assert_eq!([0x01], buf[0..1]);

        let mut writer = xrce_client_rs::micro_cdr::Encoder::new_with_endianness(&mut buf, xrce_client_rs::Endianness::BigEndianness);
        serde::Serializer::serialize_bool(&mut writer, v).unwrap() ;
        assert_eq!([0x01], buf[0..1]);
    }

}
