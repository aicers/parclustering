use crate::kdtree::KDTree;

pub fn node_distance(left: &KDTree, right: &KDTree) -> f32 {
    for d in 0..left.dim() {
        if left.get_min(d) > right.get_max(d) || right.get_min(d) > left.get_max(d) {
            let mut rsqr = 0.0;
            for dd in d..left.dim() {
                let mut tmp = f32::max(
                    left.get_min(dd) - right.get_max(dd),
                    right.get_min(dd) - left.get_max(dd),
                );
                tmp = f32::max(tmp, 0.0);
                rsqr += tmp * tmp;
            }
            return f32::sqrt(rsqr);
        }
    }
    0.0
}

#[cfg(test)]
mod tests {
    use crate::{
        node_cd::point_set_cd,
        point::Point,
        sample_points::{n_random_points, sample_points},
        wspd::geometrically_separated,
    };

    use super::*;
    #[ignore = "Checked"]
    #[test]
    fn node_dist() {
        let mut point_set: Vec<Point> = vec![
            Point {
                coords: vec![5., 1.],
            },
            Point {
                coords: vec![4., 8.],
            },
            Point {
                coords: vec![3., 7.],
            },
            Point {
                coords: vec![1., 9.],
            },
            Point {
                coords: vec![7., 3.],
            },
            Point {
                coords: vec![2., 5.],
            },
        ];
        let min_pts = 3;
        let kdtree = KDTree::build(&mut point_set);
        let left = &kdtree.left_node.as_ref().unwrap();
        let right = &kdtree.right_node.as_ref().unwrap();

        println!("{:?}", kdtree.is_leaf());
        let node_dist = node_distance(&left, &right);
        println!("{:?}", node_dist);
    }
}
