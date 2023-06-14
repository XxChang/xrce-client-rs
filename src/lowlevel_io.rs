/// A trait indicating a type that can write bytes
///
/// Note that this is similar to `std::io::Write`, but simplified for use in a
/// `no_std` context.
pub trait Write {
    /// The error type
    type Error;

    /// Writes all bytes from `data` or fails
    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Flushes all output
    fn flush(&mut self) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub struct OutOfSpace(());

#[cfg(not(feature = "std"))]
impl Write for &mut [u8] {
    type Error = OutOfSpace;

    #[inline]
    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        if data.len() > self.len() {
            return Err(OutOfSpace(()));
        }

        let (prefix, suffix) = core::mem::replace(self, &mut []).split_at_mut(data.len());
        prefix.copy_from_slice(data);
        *self = suffix;
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
