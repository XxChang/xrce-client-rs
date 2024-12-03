pub trait Clock {
    fn now(&mut self) -> i32;
}
