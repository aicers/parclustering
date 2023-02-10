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
        f.lock().unwrap().run(left, right);
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
            if left.l_max() > right.l_max() {
                find_wsp_serial(&left.left_node.as_ref().unwrap(), right, 2.0, f.clone());
                find_wsp_serial(&left.right_node.as_ref().unwrap(), right, 2.0, f.clone());
            } else {
                find_wsp_serial(&right.left_node.as_ref().unwrap(), left, 2.0, f.clone());
                find_wsp_serial(&right.right_node.as_ref().unwrap(), left, 2.0, f.clone());
            }
        }
    }
}

pub fn compute_wspd_serial<'a, T>(tree: &'a KDTree, s: &'a f64, f: Arc<Mutex<T>>)
where
    T: WspdFilter,
{
    if !tree.is_leaf() && f.lock().unwrap().start(tree) {
        if let Some(ref left_node) = tree.left_node {
            compute_wspd_serial(left_node.as_ref(), s, f.clone());
        }
        if let Some(ref right_node) = tree.right_node {
            compute_wspd_serial(right_node.as_ref(), s, f.clone());
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
    compute_wspd_serial(tree, &2., wg.clone());
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
                if left.l_max() > right.l_max() {
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
        compute_wspd_serial(tree, s, f.clone());
    }
    if !(tree.is_leaf()) && f.lock().unwrap().start(tree) {
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
    geometrically_separated(left, right, s)
}

pub fn geometrically_separated(left: &KDTree, right: &KDTree, s: f64) -> bool {
    let mut left_circle_diam: f64 = 0.0;
    let mut right_circle_diam: f64 = 0.0;
    let mut circle_distance: f64 = 0.0;
    let dimension = left.dim();
    for d in 0..dimension {
        let left_tmp_diff = left.get_max(d) - left.get_min(d);
        let right_tmp_diff = right.get_max(d) - right.get_min(d);
        let left_tmp_avg = (left.get_max(d) + left.get_min(d)) / 2.;
        let right_tmp_avg = (right.get_max(d) + right.get_min(d)) / 2.;
        circle_distance += (left_tmp_avg - right_tmp_avg).powi(2);
        left_circle_diam += left_tmp_diff.powi(2);
        right_circle_diam += right_tmp_diff.powi(2);
    }
    let left_circle_diam = left_circle_diam.sqrt();
    let right_circle_diam = right_circle_diam.sqrt();
    let my_radius: f64 = f64::max(left_circle_diam, right_circle_diam) / 2.0;
    let circle_distance = circle_distance.sqrt() - left_circle_diam / 2.0 - right_circle_diam / 2.0;

    circle_distance >= (s * my_radius)
}

//--------------------------------------------------------------------------------
//Testing the above constructed codes
//--------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::{
        node_cd::point_set_cd,
        sample_points::{n_random_points, sample_points},
    };

    use super::*;
    #[test]
    fn wpsd_set() {
        let min_pts = 3;
        let mut point_set: Vec<Point> = sample_points();

        let kdtree = KDTree::build(&mut point_set);
        println!("====================");

        let mut core_dist: Vec<f64> = point_set_cd(&point_set, &kdtree, min_pts);
        let wsp_pairs = wspd_parallel(&kdtree, 2.);

        let check =
            geometrically_separated(&kdtree.left_node.unwrap(), &kdtree.right_node.unwrap(), 2.);

        println!("{:#?}", wsp_pairs.len());
    }
}
