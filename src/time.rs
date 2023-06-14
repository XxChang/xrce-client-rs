pub trait Clock {
    fn now(&mut self) -> i64 ;
}
