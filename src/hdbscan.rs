use crate::kdtree::KDTree;
use crate::node_cd::node_cd;
use crate::point::Point;
use crate::wspdparallel::filter_wspd_paraller;

mod tests {
    use super::*;
    use rand::thread_rng;
    use rand::Rng;

    #[test]
    fn hdbscan() {
        std::env::set_var("RUST_BACKTRACE", "full");
        //Declaring initial parametres
        let mut rng = thread_rng();
        let n_random = 5;
        let min_pts = 3;

        //Generating random points for our dataset
        let mut make_random_point = || Point {
            coords: (0..2).map(|_| (rng.gen::<f64>() - 0.5) * 100.0).collect(),
        };
        let mut random_points: Vec<Point> = (0..n_random).map(|_| make_random_point()).collect();

        //Creating KD-Tree with above generated random points
        let mut kdtree = KDTree::build(&mut random_points);

        //Storing all the core distances of points in one set
        let mut core_dist: Vec<f64> = std::iter::repeat(0.).take(random_points.len()).collect();
        for (i, elem) in random_points.iter().enumerate() {
            core_dist[i] = kdtree
                .nearest_neighbours(elem, min_pts)
                .last()
                .unwrap()
                .0
                 .0;
        }
        println!("{:?}", core_dist);

        let mut cd_min = f64::MAX;
        let mut cd_max = f64::MIN;

        //println!("{:?}", kdtree);
        node_cd(&mut kdtree, &random_points, &core_dist, cd_min, cd_max);
        let beta = 2.;
        let rho_lo = 0.;
        let rho_hi = f64::MIN;

        let bccps =
            filter_wspd_paraller(&beta, &rho_lo, rho_hi, &kdtree, &core_dist, &random_points);
        println!("{:?}", kdtree);
    }
}
