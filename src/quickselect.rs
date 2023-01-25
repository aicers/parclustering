use rand::prelude::*;
use std::cmp::{Ord, Ordering};

pub fn quickselect_by<T>(
    point_list: &mut [T],
    position: usize,
    cmp: &dyn Fn(&T, &T) -> Ordering,
) -> T
where
    T: Clone,
{
    let mut pivot_index = thread_rng().gen_range(0..point_list.len());
    pivot_index = partition_by(point_list, pivot_index, &|a: &T, b: &T| cmp(a, b));

    let list_len = point_list.len();
    match position.cmp(&pivot_index) {
        Ordering::Equal => point_list[position].clone(),
        Ordering::Less => quickselect_by(&mut point_list[0..pivot_index], position, cmp),
        Ordering::Greater => quickselect_by(
            &mut point_list[pivot_index + 1..list_len],
            position - pivot_index - 1,
            cmp,
        ),
    }
}

fn partition_by<T>(
    point_list: &mut [T],
    pivot_index: usize,
    cmp: &dyn Fn(&T, &T) -> Ordering,
) -> usize {
    if point_list.len() == 1 {
        return 0;
    }
    let list_len = point_list.len();
    point_list.swap(pivot_index, list_len - 1);
    let mut store_index = 0;

    for index in 0..list_len - 1 {
        if cmp(&point_list[index], &point_list[list_len - 1]) == Ordering::Less {
            point_list.swap(index, store_index);
            store_index += 1;
        }
    }
    point_list.swap(list_len - 1, store_index);
    store_index
}
