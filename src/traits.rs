use core::fmt::Debug;

pub trait HypergraphClass: Debug + Eq {
    fn new() -> Self;
    fn is_main(&self) -> bool {
        false
    }
    fn is_sub(&self) -> bool {
        false
    }
}
