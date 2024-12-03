use crate::Result;

pub trait Transmitter {
    type Ok;

    fn send_msg(&mut self, buf: &[u8]) -> Result<Self::Ok>;
}

pub trait Receiver<'storage> {
    fn receive_msg(&'storage mut self, timeout: i32) -> Result<&'storage [u8]>;
}
