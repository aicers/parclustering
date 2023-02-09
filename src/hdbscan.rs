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
pub fn hdbscan(points: Vec<Point>, min_pts: usize) -> Vec<WeightedEdge> {
    let min_pts = min_pts;

    let mut random_points = sample_points();

    //Creating KD-Tree with above generated random points
    let mut kdtree = KDTree::build(&mut random_points);

    //Storing all the core distances of points in one set
    let mut core_dist: Vec<f64> = point_set_cd(&random_points, &kdtree, min_pts);
    //println!("{:?}", core_dist);

    let mut _cd_min = f64::MAX;
    let mut _cd_max = f64::MIN;

    node_cd(&mut kdtree, &random_points, &core_dist, _cd_min, _cd_max);
    let mut beta = 2.;
    let mut rho_lo = 0.;
    let mut rho_hi = f64::MIN;
    let mut num_edges: usize = 0;

    let mut uf = Arc::new(Mutex::new(EdgeUnionFind::new(random_points.len())));

    while uf.lock().unwrap().num_edge() < random_points.len() - 1 {
        let bccps =
            filter_wspd_paraller(&beta, &rho_lo, rho_hi, &kdtree, &core_dist, &random_points);
        //println!("{bccps:?}");
        num_edges += bccps.len();

        //println!("{uf:?}");
        println!("{num_edges}");
        println!("{rho_hi:?}");

        if bccps.len() <= 0 {
            beta *= 2.;
            rho_lo = rho_hi;
        }

        let mut edges: Vec<WEdge> = bccps
            .iter()
            .map(|bcp| {
                WEdge::new(
                    random_points.iter().position(|x| *x == bcp.u).unwrap(),
                    random_points.iter().position(|x| *x == bcp.v).unwrap(),
                    bcp.dist,
                )
            })
            .collect();

        batch_kruskal(&mut edges, random_points.len(), &mut uf);
        mark(&mut kdtree, &mut uf, &random_points);
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

        let hdbscan = hdbscan(point_set, min_pts);

        println!("{hdbscan:?}");
    }
}
