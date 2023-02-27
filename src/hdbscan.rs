use std::sync::{Arc, Mutex};
use std::time::{Instant,Duration};
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
    pub weight: f32,
}

impl WEdge {
    pub fn new(u: usize, v: usize, weight: f32) -> Self {
        Self { u, v, weight }
    }
}
pub fn hdbscan(points: &mut Vec<Point>, min_pts: usize) -> Vec<WeightedEdge> {
    let min_pts = min_pts;
    let mut wspd_time = 0.;
    //Tree construction time counter
    let const_start_time = Instant::now();
    //Creating KD-Tree with above generated random points
    let mut kdtree = KDTree::build(points);
    let const_end_time = Instant::now();
    println!("Time spent for tree construction {:?}", const_end_time.duration_since(const_start_time).as_secs_f64());
    println!("=================");

    //Storing all the core distances of points in one set
    let mut core_dist: Vec<f32> = point_set_cd(&points, &kdtree, min_pts);

    let mut _cd_min = f32::MAX;
    let mut _cd_max = f32::MIN;

    node_cd(&mut kdtree, &points, &core_dist, _cd_min, _cd_max);
    let mut beta: i64 = 2;
    let mut rho_lo = -1.;

    let mut num_edges: usize = 0;

    let mut uf = Arc::new(Mutex::new(EdgeUnionFind::new(points.len())));
    let mut i = 0;

    while uf.lock().unwrap().num_edge() < points.len() - 1 {
        let mut rho_hi = f32::MIN;
        let wspd_start = Instant::now();
        let bccps = filter_wspd_paraller(&beta, &rho_lo, &mut rho_hi, &kdtree, &core_dist, &points);
        let wspd_end = Instant::now();
        wspd_time+= wspd_end.duration_since(wspd_start).as_secs_f64();
        num_edges += bccps.len();

        if bccps.len() <= 0 {
            beta *= 2;
            rho_lo = rho_hi;
        }
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
        println!("=================");
        println!("Edges {}", num_edges);
        println!("MST Edges {:?}", uf.lock().unwrap().get_edge().len());
        println!("Rho Lo {}", rho_lo);
        println!("Rho Hi {}", rho_hi);
        println!("Beta {}", beta);
        println!("=================");
        beta *= 2;
        rho_lo = rho_hi;
    }
    println!("WSPD Construction time {:?}", wspd_time);
    let x = uf.lock().unwrap().get_edge();
    x
}
