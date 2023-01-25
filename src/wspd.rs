use crate::kdtree::KDTree;
use crate::point::Point;
use crate::wrapper::Wrapper;
use crate::wspdparallel::WspdFilter;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::cmp::max;
use std::sync::{Arc, Mutex};
use std::thread;

//Declaring the Well Separated Struct
#[derive(Debug, Clone)]
pub struct Wsp {
    pub u: KDTree,
    pub v: KDTree,
}

impl Wsp {
    pub fn new() -> Self {
        Self {
            u: KDTree::empty(),
            v: KDTree::empty(),
        }
    }

    pub fn add(left: KDTree, right: KDTree) -> Self {
        Self { u: left, v: right }
    }
}

//--------------------------------------------------------------------------------
//Computing Well Separated Pairs Sequentially
//--------------------------------------------------------------------------------
struct WspdNormalSerial {
    out: Vec<Wsp>,
}
impl WspdFilter for WspdNormalSerial {
    fn start(&mut self, tree: &KDTree) -> bool {
        true
    }
    fn run(&mut self, left: &KDTree, right: &KDTree) {
        self.out.push(Wsp::add(left.clone(), right.clone()));
    }
    fn move_on(&mut self, left: &KDTree, right: &KDTree) -> bool {
        true
    }

    fn well_separated(&mut self, left: &KDTree, right: &KDTree) -> bool {
        return geometrically_separated(left, right, 2.);
    }
}

impl WspdNormalSerial {
    fn new(out: Vec<Wsp>) -> Self {
        Self { out }
    }

    fn get_res(&mut self) -> Vec<Wsp> {
        self.out.to_vec()
    }
}

fn find_wsp_serial<'a, T>(left: &'a KDTree, right: &'a KDTree, _s: f64, f: Arc<Mutex<T>>)
where
    T: WspdFilter,
{
    if !f.lock().unwrap().move_on(left, right) {
        return;
    }

    if well_separated(left, right, 2.0) {
    } else {
        if left.is_leaf() && right.is_leaf() {
            panic!("Leaves not well separated")
        } else if left.is_leaf() {
            find_wsp_serial(&right.left_node.as_ref().unwrap(), left, 2.0, f.clone());
            find_wsp_serial(&right.right_node.as_ref().unwrap(), left, 2.0, f.clone());
        } else if right.is_leaf() {
            find_wsp_serial(&left.left_node.as_ref().unwrap(), right, 2.0, f.clone());
            find_wsp_serial(&left.right_node.as_ref().unwrap(), right, 2.0, f.clone());
        } else {
            if left.lMax() > right.lMax() {
                find_wsp_serial(&left.left_node.as_ref().unwrap(), right, 2.0, f.clone());
                find_wsp_serial(&left.right_node.as_ref().unwrap(), right, 2.0, f.clone());
            } else {
                find_wsp_serial(&right.left_node.as_ref().unwrap(), left, 2.0, f.clone());
                find_wsp_serial(&right.right_node.as_ref().unwrap(), left, 2.0, f.clone());
            }
        }
    }
}

fn computeWspdSerial<'a, T>(tree: &'a KDTree, s: &'a f64, f: Arc<Mutex<T>>)
where
    T: WspdFilter,
{
    if !tree.is_leaf() && f.lock().unwrap().start(tree) {
        if let Some(ref left_node) = tree.left_node {
            computeWspdSerial(left_node.as_ref(), s, f.clone());
        }
        if let Some(ref right_node) = tree.right_node {
            computeWspdSerial(right_node.as_ref(), s, f.clone());
        }
        let empty = KDTree::empty();
        find_wsp_serial(
            if let Some(ref left_node) = tree.left_node {
                left_node.as_ref()
            } else {
                &empty
            },
            if let Some(ref right_node) = tree.right_node {
                right_node.as_ref()
            } else {
                &empty
            },
            2.,
            f.clone(),
        );
    }
}

pub fn wspd_serial(tree: &KDTree, _s: f64) -> Vec<Wsp> {
    let out = Vec::new();
    let mut wg = Arc::new(Mutex::new(WspdNormalSerial::new(out)));
    computeWspdSerial(tree, &2., wg.clone());
    return wg.lock().unwrap().get_res();
}
//--------------------------------------------------------------------------------
//Computing Well Separated Pairs Parallel
//--------------------------------------------------------------------------------
#[derive(Debug, Clone)]
struct WspdNormalParallel {
    out: Vec<Wsp>,
}

impl WspdFilter for WspdNormalParallel {
    fn start(&mut self, tree: &KDTree) -> bool {
        true
    }

    fn run(&mut self, left: &KDTree, right: &KDTree) {
        self.out.push(Wsp::add(left.clone(), right.clone()))
    }

    fn move_on(&mut self, left: &KDTree, right: &KDTree) -> bool {
        true
    }

    fn well_separated(&mut self, left: &KDTree, right: &KDTree) -> bool {
        return geometrically_separated(left, right, 2.);
    }
}

impl WspdNormalParallel {
    fn new(out: Vec<Wsp>) -> Self {
        Self { out }
    }

    fn get_res(&self) -> Vec<Wsp> {
        self.out.to_vec()
    }
}

pub fn find_wsp_parallel<'a, T>(left: &'a KDTree, right: &'a KDTree, s: f64, f: Arc<Mutex<T>>)
where
    T: WspdFilter + std::marker::Sync + std::marker::Send,
{
    if left.size() + right.size() < 2000 {
        find_wsp_serial(left, right, 2., f);
    } else {
        if well_separated(left, right, 2.0) {
            f.lock().unwrap().run(left, right);
        } else {
            if left.is_leaf() && right.is_leaf() {
                panic!("Leaves not well separated")
            } else if left.is_leaf() {
                rayon::join(
                    || find_wsp_parallel(&right.left_node.as_ref().unwrap(), left, 2.0, f.clone()),
                    || find_wsp_parallel(&right.right_node.as_ref().unwrap(), left, 2.0, f.clone()),
                );
            } else if right.is_leaf() {
                rayon::join(
                    || find_wsp_parallel(&left.left_node.as_ref().unwrap(), right, 2.0, f.clone()),
                    || find_wsp_parallel(&left.right_node.as_ref().unwrap(), right, 2.0, f.clone()),
                );
            } else {
                if left.lMax() > right.lMax() {
                    rayon::join(
                        || {
                            find_wsp_parallel(
                                &left.left_node.as_ref().unwrap(),
                                right,
                                2.0,
                                f.clone(),
                            )
                        },
                        || {
                            find_wsp_parallel(
                                &left.right_node.as_ref().unwrap(),
                                right,
                                2.0,
                                f.clone(),
                            )
                        },
                    );
                } else {
                    rayon::join(
                        || {
                            find_wsp_parallel(
                                &right.left_node.as_ref().unwrap(),
                                left,
                                2.0,
                                f.clone(),
                            )
                        },
                        || {
                            find_wsp_parallel(
                                &right.right_node.as_ref().unwrap(),
                                left,
                                2.0,
                                f.clone(),
                            )
                        },
                    );
                }
            }
        }
    }
}

pub fn compute_wspd_parallel<'a, T>(tree: &'a KDTree, s: &'a f64, f: Arc<Mutex<T>>)
where
    T: WspdFilter + std::marker::Sync + std::marker::Send + Clone,
{
    if tree.size() < 2000 {
        computeWspdSerial(tree, s, f.clone());
    }
    if !(tree.is_leaf()) && true {
        rayon::join(
            || {
                if let Some(ref left_node) = tree.left_node {
                    compute_wspd_parallel(&left_node.as_ref(), s, f.clone())
                }
            },
            || {
                if let Some(ref right_node) = tree.right_node {
                    compute_wspd_parallel(&right_node.as_ref(), s, f.clone())
                }
            },
        );
        let empty = KDTree::empty();
        find_wsp_parallel(
            if let Some(ref left_node) = tree.left_node {
                left_node.as_ref()
            } else {
                &empty
            },
            if let Some(ref right_node) = tree.right_node {
                right_node.as_ref()
            } else {
                &empty
            },
            2.0,
            f.clone(),
        );
    }
}

pub fn wspd_parallel(tree: &KDTree, _s: f64) -> Vec<Wsp> {
    let mut wg = Arc::new(Mutex::new(WspdNormalParallel::new(Vec::with_capacity(
        tree.size(),
    ))));
    compute_wspd_parallel(tree, &_s, wg.clone());
    let res = wg.lock().unwrap().get_res();
    res
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
    let dimension = left.dim();
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
        println!("{:?}", kdtree);
        println!("====================");
        let res = if let Some(ref kdtree) = kdtree.left_node {
            println!("{:?}", kdtree);
        };

        let mut core_dist: Vec<f64> = std::iter::repeat(0.).take(wp_points.len()).collect();

        for (i, elem) in wp_points.iter().enumerate() {
            core_dist[i] = kdtree.nearest_neighbours(elem, minPts).last().unwrap().0 .0;
        }

        println!("{:?}", core_dist);
    }
}
