use std::time::Instant;

use kdtree::hdbscan::hdbscan;
use kdtree::{
    dendrogram::dendrogram,
    sample_points::{n_random_points, sample_points},
};

#[test]
fn hdbscan_test() {
    std::env::set_var("RUST_BACKTRACE", "full");
    let mut point_set = sample_points();
    let mut min_pts = 3;
    let hdbscan_start = Instant::now();
    let hdbscan = hdbscan(&mut point_set, min_pts);
    let hdbscan_end = Instant::now();
    println!("=================");
    println!("HDBSCAN Construction Time {:?}", hdbscan_end.duration_since(hdbscan_start).as_secs_f64());
}
