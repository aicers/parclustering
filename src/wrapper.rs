use std::cmp::{Ord, Ordering};
use std::ops::Neg;

#[derive(Debug, Copy, Clone)]
pub struct Wrapper(pub f64);
impl Ord for Wrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 < other.0 {
            Ordering::Less
        } else if self.0 > other.0 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl PartialOrd for Wrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl PartialEq for Wrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Wrapper {}

impl Neg for Wrapper {
    type Output = Wrapper;

    fn neg(self) -> Wrapper {
        Wrapper(-self.0)
    }
}
