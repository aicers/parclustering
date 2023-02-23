use serde::Deserialize;

use crate::wrapper::Wrapper;
use std::{
    cmp::{Ord, Ordering},
    ops::Deref,
};
#[derive(Debug, Clone, Deserialize)]
pub struct Point {
    pub coords: Vec<f32>,
}

impl Default for Point {
    fn default() -> Self {
        Self { coords: vec![0.] }
    }
}
impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut res = Ordering::Equal;
        for (x, y) in self.coords.iter().zip(other.coords.iter()) {
            res = Wrapper(*x).cmp(&Wrapper(*y));
            if res != Ordering::Equal {
                break;
            }
        }
        res
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut res = Ordering::Equal;
        for (x, y) in self.coords.iter().zip(other.coords.iter()) {
            res = Wrapper(*x).cmp(&Wrapper(*y));
            if res != Ordering::Equal {
                break;
            }
        }
        Some(res)
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.coords == other.coords
    }
}
impl Eq for Point {}

/*
impl Deref for Point {
    type Target = Point;

    fn deref(&self) -> &Self::Target {
        self
    }
}
*/
impl Point {
    pub fn distance(&self, other: &Point) -> f32 {
        self.coords
            .iter()
            .zip(other.coords.iter())
            .map(|(&x1, &x2)| (x1 - x2).powf(2.))
            .sum::<f32>()
            .sqrt()
    }
}
