use core::result::Result;
use core::fmt::Debug;

pub trait Transmitter
{
    type Error: Debug;
    type Ok;

    fn send_msg(&mut self, buf: &[u8]) -> Result<Self::Ok, Self::Error>;
}

pub trait Receiver
{
    
}