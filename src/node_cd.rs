use crate::kdtree::KDTree;
use crate::point::Point;
use crate::sample_points::sample_points;

pub fn node_cd(
    node: &mut KDTree,
    point_set: &Vec<Point>,
    core_dist: &Vec<f32>,
    cd_min: f32,
    cd_max: f32,
) {
    if node.is_leaf() {
        /*for elem in &node.points {
            println!("{:?}", elem);
            let target_cd = core_dist[point_set.iter().position(|x| x == elem).unwrap()];
            if target_cd > f64::MIN {
                node.cd_max = target_cd;
            }

            if target_cd < f64::MAX {
                node.cd_min = target_cd;
            }
        }*/
        node.cd_max = node.cd_max_calc(core_dist, point_set);
        node.cd_min = node.cd_min_calc(core_dist, point_set);
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

        node.cd_max = node.cd_max_calc(core_dist, point_set);
        node.cd_min = node.cd_min_calc(core_dist, point_set);

        /*node.cd_max = if let (Some(ref left_node), Some(ref right_node)) =
            (&node.left_node, &node.right_node)
        {
            f64::max(left_node.as_ref().cd_max, right_node.as_ref().cd_max)
        } else {
            if let Some(ref left_node) = node.left_node {
                f64::max(left_node.as_ref().cd_max, f64::MIN)
            } else if let Some(ref right_node) = node.right_node {
                f64::max(f64::MIN, right_node.as_ref().cd_max)
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
                f64::min(left_node.as_ref().cd_min, f64::MAX)
            } else if let Some(ref right_node) = node.right_node {
                f64::min(f64::MAX, right_node.as_ref().cd_min)
            } else {
                5.
            }
        };*/
    }
}

pub fn point_set_cd(point_set: &Vec<Point>, kdtree: &KDTree, min_pts: usize) -> Vec<f32> {
    let mut core_dist: Vec<f32> = std::iter::repeat(0.).take(point_set.len()).collect();
    for (i, elem) in point_set.iter().enumerate() {
        core_dist[i] = kdtree
            .nearest_neighbours(elem, min_pts)
            .last()
            .unwrap()
            .0
             .0;
    }
    core_dist
}
