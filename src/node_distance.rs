use crate::kdtree::KDTree;

pub fn node_distance(left: &KDTree, right: &KDTree) -> f64 {
    let dim = left.split_value.coords.len();

    for d in 0..dim {
        if left.get_min(d) > right.get_max(d) || right.get_min(d) > left.get_max(d) {
            let mut rsqr = 0.0;
            for dd in d..dim {
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
