use crate::kdtree::KDTree;
use crate::node_cd::node_cd;
use crate::point::Point;
use crate::wspdparallel::filter_wspd_paraller;

#[derive(Debug)]
pub struct WEdge {
    pub u: usize,
    pub v: usize,
    pub weight: f64,
}

impl WEdge {
    fn new(u: usize, v: usize, weight: f64) -> Self {
        Self { u, v, weight }
    }
}

mod tests {
    use crate::mark::mark;
    use crate::memo_gfk::batch_kruskal;
    use crate::sample_points::sample_points;
    use crate::union_find::EdgeUnionFind;

    use super::*;
    use rand::thread_rng;
    use rand::Rng;

    #[test]
    fn hdbscan() {
        std::env::set_var("RUST_BACKTRACE", "full");
        //Declaring initial parametres
        let min_pts = 3;
        /*
        let mut rng = thread_rng();
        let n_random = 50;

        //Generating random points for our dataset
        let mut make_random_point = || Point {
            coords: (0..2).map(|_| (rng.gen::<f64>() - 0.5) * 100.0).collect(),
        };
        let mut random_points: Vec<Point> = (0..n_random).map(|_| make_random_point()).collect();*/

        let mut random_points = sample_points();

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
        //println!("{:?}", core_dist);

        let mut _cd_min = f64::MAX;
        let mut _cd_max = f64::MIN;

        node_cd(&mut kdtree, &random_points, &core_dist, _cd_min, _cd_max);
        let mut beta = 2.;
        let mut rho_lo = 0.;
        let mut rho_hi = f64::MIN;
        let mut num_edges: usize = 0;

        let mut uf = EdgeUnionFind::new(random_points.len());

        while uf.num_edge() < random_points.len() - 1 {
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
        //println!("{:?}", kdtree);
    }
}
