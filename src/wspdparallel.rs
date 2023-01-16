use crate::bccp::{bcp_helper, Bcp};
use crate::kdtree::KDTree;
use crate::node_distance::node_distance;
use crate::point::Point;
use crate::wrapper::Wrapper;
use crate::wspd::{self, computeWspdParallel};
use rayon::prelude::ParallelIterator;
use rayon::{self, iter::IntoParallelIterator};
use std::cmp::max;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

pub trait WspdFilter {
    fn start(&mut self, tree: &KDTree) -> bool {
        true
    }
    fn run(&mut self, left: &KDTree, right: &KDTree) {}
    fn move_on(&mut self, left: &KDTree, right: &KDTree) -> bool {
        true
    }
    fn well_separated(&mut self, left: &KDTree, right: &KDTree) -> bool {
        true
    }
}

pub fn well_separated(left: &KDTree, right: &KDTree, s: f64) -> bool {
    unreachable(left, right)
}

pub fn unreachable(left: &KDTree, right: &KDTree) -> bool {
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
    let mut my_diam = f64::max(2.0 * my_radius, left.cd_max);
    my_diam = f64::max(my_diam, right.cd_max);

    let mut circle_distance =
        circle_distance.sqrt() - left_circle_diam.0 / 2.0 - right_circle_diam.0 / 2.0;

    let geom_separated = circle_distance >= (2.0 * my_radius);
    circle_distance = f64::max(circle_distance, left.cd_min);
    circle_distance = f64::max(circle_distance, right.cd_min);

    if circle_distance >= my_diam {
        true || geom_separated
    } else {
        false || geom_separated
    }
}
#[derive(Debug)]
pub struct RhoUpdateParallel<'a> {
    tree: &'a KDTree,
    beta: &'a f64,
    rho: Arc<Mutex<f64>>,
}

impl<'a> WspdFilter for RhoUpdateParallel<'a> {
    fn run(&mut self, left: &KDTree, right: &KDTree) {
        let mut my_dist = f64::max(node_distance(&left, &right), left.cd_min);
        my_dist = f64::max(my_dist, right.cd_min);
        self.rho = Arc::from(Mutex::from(f64::min(*self.rho.lock().unwrap(), my_dist)));
    }

    fn move_on(&mut self, left: &KDTree, right: &KDTree) -> bool {
        if left.has_id() && left.get_id() == right.get_id() {
            return false;
        }

        if left.size() + right.size() <= *self.beta as usize {
            return false;
        }

        let mut my_dist = f64::max(node_distance(left, right), left.cd_min);
        my_dist = f64::max(my_dist, right.cd_min);

        if my_dist >= *self.rho.lock().unwrap() {
            return false;
        }
        true
    }

    fn well_separated(&mut self, left: &KDTree, right: &KDTree) -> bool {
        return unreachable(left, right);
    }

    fn start(&mut self, left: &KDTree) -> bool {
        if left.size() > *self.beta as usize {
            return true;
        } else {
            return false;
        }
    }
}

impl<'a> RhoUpdateParallel<'a> {
    fn new(beta: &'a f64, tree: &'a KDTree) -> Self {
        Self {
            rho: Arc::from(Mutex::from(0.)),
            beta,
            tree,
        }
    }

    fn get_rho(&self) -> f64 {
        *self.rho.lock().unwrap()
    }
}

#[derive(Debug)]
pub struct WspdGetParallel<'a> {
    beta: &'a f64,
    rho_lo: &'a f64,
    rho_hi: &'a f64,
    tree: &'a KDTree,
    buffer: &'a mut Vec<Bcp<'a>>,
    core_dist: &'a Vec<f64>,
    point_set: &'a Vec<Point>,
    r: &'a mut Bcp<'a>,
}

impl<'a> WspdFilter for WspdGetParallel<'a> {
    fn start(&mut self, tree: &KDTree) -> bool {
        if f64::max(tree.diag(), tree.cd_max) >= *self.rho_lo {
            return true;
        } else {
            return false;
        }
    }

    fn run(&mut self, left: &KDTree, right: &KDTree) {
        vec![(left, right)].into_par_iter().for_each(|(u, v)| {
            let bcp = bcp_helper(u, v,self.r, self.core_dist, self.point_set);
            if left.size() + right.size() <= *self.beta as usize
                && bcp.dist >= *self.rho_lo
                && bcp.dist < *self.rho_hi
            {
                self.buffer.push(Bcp {
                    u: bcp.u,
                    v: bcp.v,
                    dist: bcp.dist,
                });
            }
        })
    }

    fn move_on(&mut self, left: &KDTree, right: &KDTree) -> bool {
        if left.has_id() && left.get_id() == right.get_id() {
            return false;
        }
        let mut dist = f64::max(node_distance(left, right), left.cd_min);
        dist = f64::max(dist, right.cd_min);

        if dist >= *self.rho_hi {
            return false;
        }
        dist = f64::max(node_distance(left, right), left.cd_max);
        dist = f64::max(dist, right.cd_max);

        if dist < *self.rho_lo {
            return false;
        }
        return true;
    }

    fn well_separated(&mut self, left: &KDTree, right: &KDTree) -> bool {
        return unreachable(left, right);
    }
}

impl<'a> WspdGetParallel<'a> {
    fn new(
        beta: &'a f64,
        rho_lo: &'a f64,
        rho_hi: &'a mut f64,
        tree: &'a KDTree,
        buffer: &'a mut Vec<Bcp<'a>>,
        core_dist: &'a Vec<f64>,
        point_set: &'a Vec<Point>,
        r: &'a mut Bcp<'a>,
    ) -> Self {
        Self {
            beta,
            rho_lo,
            rho_hi,
            tree,
            buffer,
            core_dist,
            point_set,
            r,
        }
    }
}

fn filter_wspd_paraller<'a>(
    beta: &'a f64,
    rho_lo: &'a f64,
    _rho_hi: &'a mut f64,
    tree: &'a KDTree,
    core_dist: &'a Vec<f64>,
    point_set: &'a Vec<Point>,
) -> Vec<Bcp<'a>> {
    let mut my_rho = RhoUpdateParallel::new(beta, tree);

    computeWspdParallel(tree.left_node.as_ref().unwrap(), &2., &mut my_rho);
    let mut rho_hi = my_rho.get_rho();
    let mut buffer: Vec<Bcp> = Vec::new();
    let mut r = Bcp::new();
    let mut my_splitter = WspdGetParallel::new(
        beta,
        rho_lo,
        &mut rho_hi,
        tree,
        &mut buffer,
        core_dist,
        point_set,
        &mut r,
    );

    computeWspdParallel(tree, &2.0, &mut my_splitter);

    let t_rho_hi = &my_rho.get_rho();
    return buffer;
}
