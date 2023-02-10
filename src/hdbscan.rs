use std::sync::{Arc, Mutex};

use crate::edge::WeightedEdge;
use crate::kdtree::KDTree;
use crate::mark::mark;
use crate::memo_gfk::batch_kruskal;
use crate::node_cd::{node_cd, point_set_cd};
use crate::point::Point;
use crate::sample_points::{n_random_points, sample_points};
use crate::union_find::EdgeUnionFind;
use crate::wspdparallel::filter_wspd_paraller;

#[derive(Debug)]
pub struct WEdge {
    pub u: usize,
    pub v: usize,
    pub weight: f64,
}

impl WEdge {
    pub fn new(u: usize, v: usize, weight: f64) -> Self {
        Self { u, v, weight }
    }
}
pub fn hdbscan(points: &mut Vec<Point>, min_pts: usize) -> Vec<WeightedEdge> {
    let min_pts = min_pts;

    //Creating KD-Tree with above generated random points
    let mut kdtree = KDTree::build(points);

    //Storing all the core distances of points in one set
    let mut core_dist: Vec<f64> = point_set_cd(&points, &kdtree, min_pts);

    let mut _cd_min = f64::MAX;
    let mut _cd_max = f64::MIN;

    node_cd(&mut kdtree, &points, &core_dist, _cd_min, _cd_max);
    let mut beta = 2.;
    let mut rho_lo = 0.;
    let mut rho_hi = f64::MIN;
    let mut num_edges: usize = 0;

    let mut uf = Arc::new(Mutex::new(EdgeUnionFind::new(points.len())));
    println!("{:?}",uf);
    while uf.lock().unwrap().num_edge() < points.len() - 1 {
        let bccps =
            filter_wspd_paraller(&beta, &rho_lo, rho_hi, &kdtree, &core_dist, &points);
        num_edges += bccps.len();
        println!("{:?}",uf.lock().unwrap().get_edge().len());
        if bccps.len() <= 0 {
            beta *= 2.;
            rho_lo = rho_hi;
        }
        println!("Next");
        println!("{:?}",uf);
        let mut edges: Vec<WEdge> = bccps
            .iter()
            .map(|bcp| {
                WEdge::new(
                    points.iter().position(|x| *x == bcp.u).unwrap(),
                    points.iter().position(|x| *x == bcp.v).unwrap(),
                    bcp.dist,
                )
            })
            .collect();

        batch_kruskal(&mut edges, points.len(), &mut uf);
        mark(&mut kdtree, &mut uf, &points);
        beta *= 2.;
        rho_lo = rho_hi;
    }
    let x = uf.lock().unwrap().get_edge();
    x
}

mod tests {
    use super::{hdbscan, *};
    use crate::sample_points::sample_points;

    #[test]
    fn hdbscan_test() {
        std::env::set_var("RUST_BACKTRACE", "full");
        let mut point_set = sample_points();
        let mut min_pts = 3;

        let hdbscan = hdbscan(&mut point_set, min_pts);

        println!("HDBSCAN {hdbscan:?}");
    }
}
