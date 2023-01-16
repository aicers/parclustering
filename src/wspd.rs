use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::kdtree::KDTree;
use crate::point::Point;
use crate::wrapper::Wrapper;
use crate::wspdparallel::WspdFilter;
use std::cmp::max;
use std::sync::{Arc, Mutex};

//Declaring the Well Separated Struct
#[derive(Debug)]
pub struct Wsp<'a> {
    pub u: &'a KDTree,
    pub v: &'a KDTree,
}

impl<'a> Wsp<'a> {
    pub fn new() -> Self {
        let left = KDTree::empty();
        let right = KDTree::empty();
        Self {
            u: &left,
            v: &right,
        }
    }

    pub fn add(left: &'a KDTree, right: &'a KDTree) -> Self {
        Self { u: left, v: right }
    }
}

//--------------------------------------------------------------------------------
//Computing Well Separated Pairs Sequentially
//--------------------------------------------------------------------------------
struct WspdNormalSerial<'a> {
    out: &'a mut Vec<Wsp<'a>>,
}
impl<'a> WspdFilter for WspdNormalSerial<'a> {
    fn start(&mut self, tree: &KDTree) -> bool {
        true
    }
    fn run(&mut self, left: &KDTree, right: &KDTree) {
        self.out.push(Wsp::add(left, right));
    }
    fn move_on(&mut self, left: &KDTree, right: &KDTree) -> bool {
        true
    }

    fn well_separated(&mut self, left: &KDTree, right: &KDTree) -> bool {
        return geometrically_separated(left, right, 2.);
    }
}

impl<'a> WspdNormalSerial<'a> {
    fn new(out: &'a mut Vec<Wsp<'a>>) -> Self {
        Self { out }
    }
}

fn find_wsp_serial<'a, T>(left: &'a KDTree, right: &'a KDTree, s: f64, f: &'a mut T)
where
    T: WspdFilter,
{
    if !f.move_on(left, right) {
        return;
    }

    if well_separated(left, right, 2.0) {
    } else {
        if left.is_leaf() && right.is_leaf() {
            panic!("Leaves not well separated")
        } else if left.is_leaf() {
            find_wsp_serial(&right.left_node.as_ref().unwrap(), left, 2.0, f);
            find_wsp_serial(&right.right_node.as_ref().unwrap(), left, 2.0, f);
        } else if right.is_leaf() {
            find_wsp_serial(&left.left_node.as_ref().unwrap(), right, 2.0, f);
            find_wsp_serial(&left.right_node.as_ref().unwrap(), right, 2.0, f);
        } else {
            if left.lMax() > right.lMax() {
                find_wsp_serial(&left.left_node.as_ref().unwrap(), right, 2.0, f);
                find_wsp_serial(&left.right_node.as_ref().unwrap(), right, 2.0, f);
            } else {
                find_wsp_serial(&right.left_node.as_ref().unwrap(), left, 2.0, f);
                find_wsp_serial(&right.right_node.as_ref().unwrap(), left, 2.0, f);
            }
        }
    }
}

fn computeWspdSerial<'a, T>(tree: &'a KDTree, s: &'a f64, f: &'a mut T)
where
    T: WspdFilter,
{
    if !tree.is_leaf() && f.start(tree) {
        computeWspdSerial(tree.left_node.as_ref().unwrap(), s, f);
        computeWspdSerial(tree.right_node.as_ref().unwrap(), s, f);

        find_wsp_serial(
            tree.left_node.as_ref().unwrap(),
            tree.right_node.as_ref().unwrap(),
            2.,
            f,
        );
    }
}

pub fn wspd_serial(tree: &KDTree, s: f64) -> Vec<Wsp> {
    let mut out = Vec::new();
    let mut wg = WspdNormalSerial::new(&mut out);
    computeWspdSerial(tree, &2., &mut wg);
    out
}
//--------------------------------------------------------------------------------
//Computing Well Separated Pairs Parallel
//--------------------------------------------------------------------------------
struct WspdNormalParallel<'a> {
    out: &'a mut Vec<Wsp<'a>>,
}

impl<'a> WspdFilter for WspdNormalParallel<'a> {
    fn start(&mut self, tree: &KDTree) -> bool {
        true
    }

    fn run(&mut self, left: &KDTree, right: &KDTree) {
        vec![(left, right)]
            .into_par_iter()
            .for_each(|(u, v)| self.out.push(Wsp { u: left, v: right }))
    }

    fn move_on(&mut self, left: &KDTree, right: &KDTree) -> bool {
        true
    }

    fn well_separated(&mut self, left: &KDTree, right: &KDTree) -> bool {
        return geometrically_separated(left, right, 2.);
    }
}

impl<'a> WspdNormalParallel<'a> {
    fn new(&self, n: usize) -> Self {
        let mut out: Vec<Wsp> = Vec::with_capacity(n);
        Self { out: &mut out }
    }

    fn get_res(&self) -> &mut Vec<Wsp<'a>> {
        self.out
    }
}

pub fn find_wsp_parallel<'a, T>(left: &'a KDTree, right: &'a KDTree, s: f64, f: &'a mut T)
where
    T: WspdFilter + std::marker::Sync + std::marker::Send,
{
    if left.points.len() + right.points.len() < 2000 {
        find_wsp_serial(left, right, 2., f);
    } else {
        if well_separated(left, right, 2.0) {
            f.run(left, right);
        } else {
            if left.is_leaf() && right.is_leaf() {
                panic!("Leaves not well separated")
            } else if left.is_leaf() {
                rayon::join(
                    || find_wsp_parallel(&right.left_node.as_ref().unwrap(), left, 2.0, f),
                    || find_wsp_parallel(&right.right_node.as_ref().unwrap(), left, 2.0, f),
                );
            } else if right.is_leaf() {
                rayon::join(
                    || find_wsp_parallel(&left.left_node.as_ref().unwrap(), right, 2.0, f),
                    || find_wsp_parallel(&left.right_node.as_ref().unwrap(), right, 2.0, f),
                );
            } else {
                if left.lMax() > right.lMax() {
                    rayon::join(
                        || find_wsp_parallel(&left.left_node.as_ref().unwrap(), right, 2.0, f),
                        || find_wsp_parallel(&left.right_node.as_ref().unwrap(), right, 2.0, f),
                    );
                } else {
                    rayon::join(
                        || find_wsp_parallel(&right.left_node.as_ref().unwrap(), left, 2.0, f),
                        || find_wsp_parallel(&right.right_node.as_ref().unwrap(), left, 2.0, f),
                    );
                }
            }
        }
    }
}

pub fn computeWspdParallel<'a, T>(tree: &'a KDTree, s: &'a f64, f: &'a mut T)
where
    T: WspdFilter + std::marker::Sync + std::marker::Send,
{
    if tree.size() < 2000 {
        computeWspdSerial(tree, s, f);
    }
    if !(tree.is_leaf()) && true {
        rayon::join(
            || computeWspdParallel(&tree.left_node.as_ref().unwrap(), s, f),
            || computeWspdParallel(&tree.right_node.as_ref().unwrap(), s, f),
        );
        find_wsp_parallel(
            tree.left_node.as_ref().unwrap(),
            tree.right_node.as_ref().unwrap(),
            2.0,
            f,
        );
    }
}

//--------------------------------------------------------------------------------
//Checking Node pairs for "Geometrically Separated" condition
//--------------------------------------------------------------------------------

pub fn well_separated(left: &KDTree, right: &KDTree, s: f64) -> bool {
    geometrically_separated(left, right, 2.0)
}

pub fn geometrically_separated(left: &KDTree, right: &KDTree, s: f64) -> bool {
    let mut left_circle_diam: f64 = 0.0;
    let mut right_circle_diam: f64 = 0.0;
    let mut circle_distance: f64 = 0.0;
    let dimension = left.points[0].coords.len();
    for d in 0..dimension {
        let left_tmp_diff = left.get_max(d) - left.get_min(d);
        let right_tmp_diff = right.get_max(d) - right.get_min(d);
        let left_tmp_avg = (left.get_max(d) + left.get_min(d)).powi(2);
        let right_tmp_avg = (right.get_max(d) - right.get_min(d)).powi(2);
        circle_distance += (left_tmp_avg - right_tmp_avg).powi(2);
        left_circle_diam += left_tmp_diff.powi(2);
        right_circle_diam += right_tmp_diff.powi(2)
    }
    let left_circle_diam = Wrapper(left_circle_diam.sqrt());
    let right_circle_diam = Wrapper(right_circle_diam.sqrt());
    let my_radius: f64 = max(left_circle_diam, right_circle_diam).0 / 2.0;
    let circle_distance =
        circle_distance.sqrt() - left_circle_diam.0 / 2.0 - right_circle_diam.0 / 2.0;

    circle_distance >= (s * my_radius)
}

//--------------------------------------------------------------------------------
//Testing the above constructed codes
//--------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn core_distance() {
        let minPts = 3;
        let mut wp_points: Vec<Point> = [
            [2.0, 3.0],
            [5.0, 4.0],
            [9.0, 6.0],
            [4.0, 7.0],
            [8.0, 1.0],
            [7.0, 2.0],
        ]
        .iter()
        .map(|x| Point { coords: x.to_vec() })
        .collect();

        let kdtree = KDTree::build(&mut wp_points);

        let mut core_dist: Vec<f64> = std::iter::repeat(0.).take(wp_points.len()).collect();

        for (i, elem) in wp_points.iter().enumerate() {
            core_dist[i] = kdtree.nearest_neighbours(elem, minPts).last().unwrap().0 .0;
        }

        println!("{:?}", core_dist);
    }
}
