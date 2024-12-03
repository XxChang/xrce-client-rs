use super::SerialPlatformOps;
use crate::communication::{Receiver, Transmitter};
use crate::time::Clock;
use crate::{Error, Result};

const FRAMING_BEGIN_FLAG: u8 = 0x7E;
const FRAMING_ESC_FLAG: u8 = 0x7D;
const FRAMING_XOR_FLAG: u8 = 0x20;
const UXR_SERIAL_TRANSPORT_MTU: usize = 512;

// CRC-16 table for POLY 0x8005 (x^16 + x^15 + x^2 + 1).
const CRC16_TABLE: [u16; 256] = [
    0x0000, 0xC0C1, 0xC181, 0x0140, 0xC301, 0x03C0, 0x0280, 0xC241, 0xC601, 0x06C0, 0x0780, 0xC741,
    0x0500, 0xC5C1, 0xC481, 0x0440, 0xCC01, 0x0CC0, 0x0D80, 0xCD41, 0x0F00, 0xCFC1, 0xCE81, 0x0E40,
    0x0A00, 0xCAC1, 0xCB81, 0x0B40, 0xC901, 0x09C0, 0x0880, 0xC841, 0xD801, 0x18C0, 0x1980, 0xD941,
    0x1B00, 0xDBC1, 0xDA81, 0x1A40, 0x1E00, 0xDEC1, 0xDF81, 0x1F40, 0xDD01, 0x1DC0, 0x1C80, 0xDC41,
    0x1400, 0xD4C1, 0xD581, 0x1540, 0xD701, 0x17C0, 0x1680, 0xD641, 0xD201, 0x12C0, 0x1380, 0xD341,
    0x1100, 0xD1C1, 0xD081, 0x1040, 0xF001, 0x30C0, 0x3180, 0xF141, 0x3300, 0xF3C1, 0xF281, 0x3240,
    0x3600, 0xF6C1, 0xF781, 0x3740, 0xF501, 0x35C0, 0x3480, 0xF441, 0x3C00, 0xFCC1, 0xFD81, 0x3D40,
    0xFF01, 0x3FC0, 0x3E80, 0xFE41, 0xFA01, 0x3AC0, 0x3B80, 0xFB41, 0x3900, 0xF9C1, 0xF881, 0x3840,
    0x2800, 0xE8C1, 0xE981, 0x2940, 0xEB01, 0x2BC0, 0x2A80, 0xEA41, 0xEE01, 0x2EC0, 0x2F80, 0xEF41,
    0x2D00, 0xEDC1, 0xEC81, 0x2C40, 0xE401, 0x24C0, 0x2580, 0xE541, 0x2700, 0xE7C1, 0xE681, 0x2640,
    0x2200, 0xE2C1, 0xE381, 0x2340, 0xE101, 0x21C0, 0x2080, 0xE041, 0xA001, 0x60C0, 0x6180, 0xA141,
    0x6300, 0xA3C1, 0xA281, 0x6240, 0x6600, 0xA6C1, 0xA781, 0x6740, 0xA501, 0x65C0, 0x6480, 0xA441,
    0x6C00, 0xACC1, 0xAD81, 0x6D40, 0xAF01, 0x6FC0, 0x6E80, 0xAE41, 0xAA01, 0x6AC0, 0x6B80, 0xAB41,
    0x6900, 0xA9C1, 0xA881, 0x6840, 0x7800, 0xB8C1, 0xB981, 0x7940, 0xBB01, 0x7BC0, 0x7A80, 0xBA41,
    0xBE01, 0x7EC0, 0x7F80, 0xBF41, 0x7D00, 0xBDC1, 0xBC81, 0x7C40, 0xB401, 0x74C0, 0x7580, 0xB541,
    0x7700, 0xB7C1, 0xB681, 0x7640, 0x7200, 0xB2C1, 0xB381, 0x7340, 0xB101, 0x71C0, 0x7080, 0xB041,
    0x5000, 0x90C1, 0x9181, 0x5140, 0x9301, 0x53C0, 0x5280, 0x9241, 0x9601, 0x56C0, 0x5780, 0x9741,
    0x5500, 0x95C1, 0x9481, 0x5440, 0x9C01, 0x5CC0, 0x5D80, 0x9D41, 0x5F00, 0x9FC1, 0x9E81, 0x5E40,
    0x5A00, 0x9AC1, 0x9B81, 0x5B40, 0x9901, 0x59C0, 0x5880, 0x9841, 0x8801, 0x48C0, 0x4980, 0x8941,
    0x4B00, 0x8BC1, 0x8A81, 0x4A40, 0x4E00, 0x8EC1, 0x8F81, 0x4F40, 0x8D01, 0x4DC0, 0x4C80, 0x8C41,
    0x4400, 0x84C1, 0x8581, 0x4540, 0x8701, 0x47C0, 0x4680, 0x8641, 0x8201, 0x42C0, 0x4380, 0x8341,
    0x4100, 0x81C1, 0x8081, 0x4040,
];

fn uxr_update_crc(crc: &mut u16, data: u8) {
    let crc_bytes = crc.to_le_bytes();
    *crc = (*crc >> 8) ^ CRC16_TABLE[(crc_bytes[0] ^ data) as usize];
}

enum FramingInputState {
    FramingUninitialized,
    FramingReadingSrcAddr,
    FramingReadingDstAddr,
    FramingReadingLenLSB,
    FramingReadingLenMSB,
    FramingReadingPayload,
    FramingReadingCrcLSB,
    FramingReadingCrcMSB,
}

pub struct FramingIO {
    state: FramingInputState,
    local_addr: u8,
    rb: [u8; 42],
    rb_head: usize,
    rb_tail: usize,
    src_addr: u8,
    wb: [u8; 42],
    wb_pos: usize,
}

impl FramingIO {
    pub fn new(local_addr: u8) -> Self {
        FramingIO {
            state: FramingInputState::FramingUninitialized,
            src_addr: 0,
            local_addr,
            rb: [0u8; 42],
            rb_head: 0,
            rb_tail: 0,
            wb: [0u8; 42],
            wb_pos: 0,
        }
    }
}

pub struct SerialTransport<Comm: SerialPlatformOps> {
    buffer: [u8; UXR_SERIAL_TRANSPORT_MTU],
    framing_io: FramingIO,
    remote_addr: u8,
    platform: Comm,
}

impl<Comm: SerialPlatformOps> SerialTransport<Comm> {
    pub fn new(platform: Comm, remote_addr: u8, local_addr: u8) -> Self {
        SerialTransport {
            buffer: [0u8; UXR_SERIAL_TRANSPORT_MTU],
            framing_io: FramingIO::new(local_addr),
            remote_addr,
            platform,
        }
    }

    fn framing_write_transport(&mut self) -> Result<usize> {
        let mut bytes_written: usize = 0;

        loop {
            let last_written = self.platform.write_serial_data(
                &self.framing_io.wb[bytes_written..(self.framing_io.wb_pos - bytes_written)],
            )?;
            bytes_written += last_written;
            if !((bytes_written < self.framing_io.wb_pos) && (0 < last_written)) {
                break;
            }
        }

        if bytes_written == self.framing_io.wb_pos {
            self.framing_io.wb_pos = 0;
            Ok(bytes_written)
        } else {
            Err(Error::PartWritten(bytes_written))
        }
    }

    fn get_next_octet(&mut self) -> Option<u8> {
        if self.framing_io.rb_head != self.framing_io.rb_tail {
            if FRAMING_ESC_FLAG != self.framing_io.rb[self.framing_io.rb_tail] {
                let octet = self.framing_io.rb[self.framing_io.rb_tail];
                self.framing_io.rb_tail =
                    (self.framing_io.rb_tail + 1) % core::mem::size_of_val(&self.framing_io.rb);
                if octet != FRAMING_BEGIN_FLAG {
                    Some(octet)
                } else {
                    None
                }
            } else {
                let temp_tail =
                    (self.framing_io.rb_tail + 1) % core::mem::size_of_val(&self.framing_io.rb);
                if temp_tail != self.framing_io.rb_head {
                    let mut octet = self.framing_io.rb[temp_tail];
                    self.framing_io.rb_tail =
                        (self.framing_io.rb_tail + 2) % core::mem::size_of_val(&self.framing_io.rb);
                    if octet != FRAMING_BEGIN_FLAG {
                        octet ^= FRAMING_XOR_FLAG;
                        Some(octet)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        } else {
            None
        }
    }

    fn add_nex_octet(&mut self, octet: u8) -> bool {
        if (octet == FRAMING_BEGIN_FLAG) || (FRAMING_ESC_FLAG == octet) {
            // byte stuffing
            if (self.framing_io.wb_pos + 1) < core::mem::size_of_val(&self.framing_io.wb) {
                self.framing_io.wb[self.framing_io.wb_pos] = FRAMING_ESC_FLAG;
                self.framing_io.wb[self.framing_io.wb_pos + 1] = octet ^ FRAMING_XOR_FLAG;
                self.framing_io.wb_pos = self.framing_io.wb_pos + 2;
                true
            } else {
                false
            }
        } else {
            if self.framing_io.wb_pos < core::mem::size_of_val(&self.framing_io.wb) {
                self.framing_io.wb[self.framing_io.wb_pos] = octet;
                self.framing_io.wb_pos = self.framing_io.wb_pos + 1;
                true
            } else {
                false
            }
        }
    }

    fn write_framed_msg(&mut self, buf: &[u8], remote_addr: u8) -> Result<usize> {
        self.framing_io.wb[0] = FRAMING_BEGIN_FLAG;
        self.framing_io.wb_pos = 1;

        self.add_nex_octet(self.framing_io.local_addr);
        self.add_nex_octet(remote_addr);
        let len = buf.len().to_le_bytes();
        self.add_nex_octet(len[0]);
        self.add_nex_octet(len[1]);

        let mut crc: u16 = 0;
        let mut written_len: usize = 0;
        while written_len < buf.len() {
            let octet = buf[written_len];
            if self.add_nex_octet(octet) {
                written_len += 1;
                uxr_update_crc(&mut crc, octet);
            } else {
                self.framing_write_transport().map_err(|e| match e {
                    Error::PartWritten(_) => Error::PartWritten(written_len),
                    _ => e,
                })?;
            }
        }

        if 0 < self.framing_io.wb_pos {
            self.framing_write_transport().map_err(|e| match e {
                Error::PartWritten(_) => Error::PartWritten(written_len),
                _ => e,
            })?;
        }

        Ok(buf.len())
    }

    fn framing_read_transport(&mut self, timeout: &mut i32, max_size: usize) -> Result<usize> {
        let time_init = self.platform.millis();

        // let mut av_len = [0;2];
        let mut av_len = if self.framing_io.rb_head == self.framing_io.rb_tail {
            self.framing_io.rb_head = 0;
            self.framing_io.rb_tail = 0;
            (core::mem::size_of_val(&self.framing_io.rb) - 1, 0)
        } else if self.framing_io.rb_head > self.framing_io.rb_tail {
            if 0 < self.framing_io.rb_tail {
                (
                    core::mem::size_of_val(&self.framing_io.rb) - self.framing_io.rb_head,
                    self.framing_io.rb_tail - 1,
                )
            } else {
                (
                    core::mem::size_of_val(&self.framing_io.rb) - self.framing_io.rb_head - 1,
                    0,
                )
            }
        } else {
            (self.framing_io.rb_tail - self.framing_io.rb_head - 1, 0)
        };

        let mut bytes_read: [usize; 2] = [0; 2];

        if max_size < av_len.0 {
            av_len.0 = max_size;
            av_len.1 = 0;
        } else if max_size < (av_len.0 + av_len.1) {
            av_len.1 = max_size - av_len.0;
        }

        if 0 < av_len.0 {
            let timeout_temp = *timeout;
            // !todo
            bytes_read[0] = self.platform.read_serial_data(
                &mut self.framing_io.rb[self.framing_io.rb_head..],
                av_len.0,
                timeout_temp,
            )?;
            self.framing_io.rb_head = (self.framing_io.rb_head + bytes_read[0])
                % core::mem::size_of_val(&self.framing_io.rb);
            if 0 < bytes_read[0] {
                if (bytes_read[0] == av_len.0) && (0 < av_len.1) {
                    bytes_read[1] = self.platform.read_serial_data(
                        &mut self.framing_io.rb[self.framing_io.rb_head..],
                        av_len.1,
                        0,
                    )?;
                    self.framing_io.rb_head = (self.framing_io.rb_head + bytes_read[1])
                        % core::mem::size_of_val(&self.framing_io.rb);
                }
            }
        }

        *timeout -= (self.platform.millis() - time_init);
        *timeout = if 0 > *timeout { 0 } else { *timeout };
        Ok(bytes_read[0] + bytes_read[1])
    }

    fn read_framed_msg(&mut self, timeout: &mut i32) -> Result<(usize, u8)> {
        let mut rv = 0;
        use FramingInputState::*;
        if self.framing_io.rb_head == self.framing_io.rb_tail {
            self.framing_read_transport(timeout, 5)?;
        }

        if self.framing_io.rb_tail != self.framing_io.rb_head {
            'outer: loop {
                match self.framing_io.state {
                    FramingUninitialized => {
                        let mut octet = 0;
                        while (FRAMING_BEGIN_FLAG != octet)
                            && (self.framing_io.rb_head != self.framing_io.rb_tail)
                        {
                            octet = self.framing_io.rb[self.framing_io.rb_tail];
                            self.framing_io.rb_tail = (self.framing_io.rb_tail + 1)
                                % core::mem::size_of_val(&self.framing_io.rb);
                        }

                        if FRAMING_BEGIN_FLAG == octet {
                            self.framing_io.state = FramingReadingSrcAddr;
                        } else {
                            break 'outer;
                        }
                    }
                    FramingReadingSrcAddr => {
                        if let Some(octet) = self.get_next_octet() {
                            self.framing_io.src_addr = octet;
                            self.framing_io.state = FramingReadingDstAddr;
                        } else if 0 < self.framing_read_transport(timeout, 4)? {
                        } else {
                            if FRAMING_BEGIN_FLAG != FRAMING_BEGIN_FLAG {
                                break 'outer;
                            }
                        }
                    }
                    FramingReadingDstAddr => {}
                    FramingReadingLenLSB => {}
                    FramingReadingLenMSB => {}
                    FramingReadingPayload => {}
                    FramingReadingCrcLSB => {}
                    FramingReadingCrcMSB => {}
                }
            }
        };

        Ok((rv, self.framing_io.src_addr))
    }
}

impl<Comm: SerialPlatformOps> Transmitter for SerialTransport<Comm> {
    type Ok = usize;

    fn send_msg(&mut self, buf: &[u8]) -> Result<Self::Ok> {
        self.write_framed_msg(buf, self.remote_addr)
    }
}

impl<'storage, Comm: SerialPlatformOps> Receiver<'storage> for SerialTransport<Comm> {
    fn receive_msg(&'storage mut self, timeout: i32) -> Result<&'storage [u8]> {
        let mut timeout = timeout;

        loop {
            let (bytes_read, remote_addr) = self.read_framed_msg(&mut timeout)?;

            if (timeout < 0) || (bytes_read != 0) {
                if remote_addr == self.remote_addr {
                    return Ok(&self.buffer[..bytes_read]);
                } else {
                    return Err(Error::RemoteAddrError);
                }
            }
        }
    }
}

impl<Comm: SerialPlatformOps> Clock for SerialTransport<Comm> {
    fn now(&mut self) -> i32 {
        self.platform.millis()
    }
}
