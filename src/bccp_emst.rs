use crate::kdtree::KDTree;
use crate::node_distance::node_distance;
use crate::point::Point;
use crate::wrapper::Wrapper;

#[derive(Debug, Clone)]
pub struct Bcp {
    pub u: Point,
    pub v: Point,
    pub dist: f64,
}

impl Bcp {
    pub fn new() -> Self {
        Self {
            u: Point { coords: vec![0.] },
            v: Point { coords: vec![0.] },
            dist: std::f64::MAX,
        }
    }

    pub fn update(&mut self, u: Point, v: Point, dist: f64) {
        if dist < self.dist {
            self.u = u;
            self.v = v;
            self.dist = dist;
        }
    }
}
//===================================================
// HDBSCAN Bichromatic Closest Pair Calculation    ##
//===================================================
pub fn bcp_helper<'a>(
    left: &'a KDTree,
    right: &'a KDTree,
    r: &'a mut Bcp,
    core_dist: &'a Vec<f64>,
    point_set: &'a Vec<Point>,
) {
    if node_distance(left, right) > r.dist {
        return;
    }
    if left.is_leaf() && right.is_leaf() {
        for i in 0..left.size() {
            for j in 0..right.size() {
                
                r.update(left.points[i].clone(), right.points[j].clone(), left.points[i].distance(&right.points[j]));
            }
        }
    } else {
        if left.is_leaf() {
            if node_distance(left, &right.left_node.as_ref().unwrap())
                < node_distance(left, &right.right_node.as_ref().unwrap())
            {
                if let Some(ref left_node) = right.left_node {
                    bcp_helper(left, &left_node.as_ref(), r, core_dist, point_set);
                }
            } else {
                if let Some(ref right_node) = right.right_node {
                    bcp_helper(left, &right_node.as_ref(), r, core_dist, point_set);
                }
            }
        } else if right.is_leaf() {
            if node_distance(right, &left.left_node.as_ref().unwrap())
                < node_distance(right, &left.right_node.as_ref().unwrap())
            {
                if let Some(ref left_node) = left.left_node {
                    bcp_helper(&left_node, right, r, core_dist, point_set);
                }
            } else {
                if let Some(ref right_node) = left.right_node {
                    bcp_helper(&right_node, right, r, core_dist, point_set);
                }
            }
        } else {
            let mut ordering: [(&KDTree, &KDTree); 4] = [
                (
                    &left.right_node.as_ref().unwrap(),
                    &right.left_node.as_ref().unwrap(),
                ),
                (
                    &left.left_node.as_ref().unwrap(),
                    &right.left_node.as_ref().unwrap(),
                ),
                (
                    &left.left_node.as_ref().unwrap(),
                    &right.right_node.as_ref().unwrap(),
                ),
                (
                    &left.right_node.as_ref().unwrap(),
                    &right.right_node.as_ref().unwrap(),
                ),
            ];
            ordering.sort_by(|a, b| {
                let distance_a = node_distance(a.0, a.1);
                let distance_b = node_distance(b.0, b.1);
                Wrapper(distance_a).cmp(&Wrapper(distance_b))
            });
            for (i, j) in ordering.iter() {
                bcp_helper(i, j, r, core_dist, point_set);
            }
        }
    }
}

//===================================================
// Brute Force Bichromatic Closest Pair Calculation##
//===================================================
pub fn brute_force_bcp<'a>(
    left: &'a KDTree,
    right: &'a KDTree,
    core_dist: &'a Vec<f64>,
    point_set: &'a Vec<Point>,
) -> Bcp {
    let mut r = Bcp::new();
    for i in 0..left.points.len() {
        for j in 0..right.points.len() {
            let mut dist = f64::max(
                (left.points[i]).distance(&right.points[j]),
                core_dist[point_set.iter().position(|x| x == &left.points[i]).unwrap()],
            );
            dist = f64::max(
                dist,
                core_dist[point_set
                    .iter()
                    .position(|x| x == &right.points[j])
                    .unwrap()],
            );
            r.update(left.points[i].clone(), right.points[j].clone(), dist);
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use crate::{
        node_cd::point_set_cd,
        sample_points::{n_random_points, sample_points},
    };

    use super::*;

    #[test]
    fn bccp_emst() {
        let mut point_set: Vec<Point> = sample_points();
        let min_pts = 3;
        let kdtree = KDTree::build(&mut point_set);
        let point_set_cd = point_set_cd(&point_set, &kdtree, min_pts);
        let mut r = Bcp::new();
        let left = kdtree.left_node.unwrap();
        let right = kdtree.right_node.unwrap();
        let bccps = bcp_helper(&right, &right, &mut r, &point_set_cd, &point_set);
        let brute_f = brute_force_bcp(&left, &right, &point_set_cd, &point_set);

        //assert_eq!(r.u, brute_f.u);
        //assert_eq!(r.v, brute_f.v);
        assert_eq!(r.dist, brute_f.dist);
    }
}
