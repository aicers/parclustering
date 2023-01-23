use crate::kdtree::KDTree;
use crate::point::Point;

pub fn node_cd(
    node: &mut KDTree,
    point_set: &Vec<Point>,
    core_dist: &Vec<f64>,
    cd_min: f64,
    cd_max: f64,
) {
    if node.is_leaf() {
        for elem in &node.points {
            println!("{:?}", elem);
            let target_cd = core_dist[point_set.iter().position(|x| x == elem).unwrap()];
            if target_cd > cd_max {
                node.cd_max = target_cd;
            }

            if target_cd < cd_min {
                node.cd_min = target_cd;
            }
        }
    } else {
        if node.size() > 2000 {
            rayon::join(
                || {
                    if let Some(ref mut left_node) = node.left_node {
                        node_cd(left_node.as_mut(), point_set, core_dist, cd_min, cd_max);
                    }
                },
                || {
                    if let Some(ref mut right_node) = node.right_node {
                        node_cd(right_node.as_mut(), point_set, core_dist, cd_min, cd_max);
                    }
                },
            );
        } else {
            if let Some(ref mut left_node) = node.left_node {
                node_cd(left_node.as_mut(), point_set, core_dist, cd_min, cd_max);
            }
            if let Some(ref mut right_node) = node.right_node {
                node_cd(right_node.as_mut(), point_set, core_dist, cd_min, cd_max);
            }
        };

        node.cd_max = if let (Some(ref left_node), Some(ref right_node)) =
            (&node.left_node, &node.right_node)
        {
            f64::max(left_node.as_ref().cd_max, right_node.as_ref().cd_max)
        } else {
            if let Some(ref left_node) = node.left_node {
                f64::max(left_node.as_ref().cd_max, 0.0)
            } else if let Some(ref right_node) = node.right_node {
                f64::max(0.0, right_node.as_ref().cd_max)
            } else {
                1.
            }
        };

        node.cd_min = if let (Some(ref left_node), Some(ref right_node)) =
            (&node.left_node, &node.right_node)
        {
            f64::max(left_node.as_ref().cd_min, right_node.as_ref().cd_min)
        } else {
            if let Some(ref left_node) = node.left_node {
                f64::min(left_node.as_ref().cd_min, 0.0)
            } else if let Some(ref right_node) = node.right_node {
                f64::min(0.0, right_node.as_ref().cd_min)
            } else {
                5.
            }
        };
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use rand::Rng;
    #[ignore = "Not complete yet"]
    #[test]
    fn hdbscan() {
        let mut rng = thread_rng();
        let n_random = 1_000_000;
        let mut make_random_point = || Point {
            coords: (0..100)
                .map(|_| (rng.gen::<f64>() - 0.5) * 1000000.0)
                .collect(),
        };
        let mut random_points: Vec<Point> = (0..n_random).map(|_| make_random_point()).collect();
        let _search_point = random_points[0].clone();

        let _kdtree = KDTree::build(&mut random_points);
        //let closest_pts = kdtree.nearest_neighbours(&search_point, 4);
    }
}
