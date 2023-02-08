use crate::kdtree::KDTree;

pub fn node_distance(left: &KDTree, right: &KDTree) -> f64 {
    for d in 0..left.dim() {
        if left.get_min(d) > right.get_max(d) || right.get_min(d) > left.get_max(d) {
            let mut rsqr = 0.0;
            for dd in d..left.dim() {
                let mut tmp = f64::max(
                    left.get_min(dd) - right.get_max(dd),
                    right.get_min(dd) - left.get_max(dd),
                );
                tmp = f64::max(tmp, 0.0);
                rsqr += tmp * tmp;
            }
            return f64::sqrt(rsqr);
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
    };

    use super::*;
    #[ignore = "Checked"]
    #[test]
    fn node_dist() {
        let mut point_set: Vec<Point> = sample_points();
        let min_pts = 3;
        let kdtree = KDTree::build(&mut point_set);
        let left = kdtree.left_node.unwrap();
        let right = kdtree.right_node.unwrap();

        let node_dist = node_distance(&left, &right);

        println!("{node_dist:?}");
    }
}
