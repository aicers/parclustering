use crate::kdtree::KDTree;
use crate::node_distance::node_distance;
use crate::point::Point;
use crate::wrapper::Wrapper;
#[derive(Debug, Clone, Copy)]
pub struct Bcp<'a> {
    pub u: &'a Point,
    pub v: &'a Point,
    pub dist: f64,
}

impl<'a> Bcp<'a> {
    pub fn new() -> Self {
        Self {
            u: &Point { coords: vec![0.] },
            v: &Point { coords: vec![0.] },
            dist: std::f64::MAX,
        }
    }

    pub fn update(&mut self, u: &'a Point, v: &'a Point, dist: f64) {
        if dist < self.dist {
            self.u = u;
            self.v = v;
            self.dist = dist;
        }
    }
}

pub fn bcp_helper<'a>(
    left: &KDTree,
    right: &KDTree,
    r: &mut Bcp,
    coreDist: &Vec<f64>,
    point_set: &Vec<Point>,
) -> &'a mut Bcp {
    if left.is_leaf() && right.is_leaf() {
        for i in 0..left.points.len() {
            for j in 0..right.points.len() {
                let mut dist = f64::max(
                    (left.points[i]).distance(&right.points[j]),
                    coreDist[point_set.iter().position(|x| x == &left.points[i]).unwrap()],
                );
                dist = f64::max(
                    dist,
                    coreDist[point_set
                        .iter()
                        .position(|x| x == &right.points[j])
                        .unwrap()],
                );
                r.update(&left.points[i], &right.points[j], dist);
            }
        }
    } else {
        if left.is_leaf() {
            if node_distance(left, &right.left_node.as_ref().unwrap())
                < node_distance(left, &right.right_node.as_ref().unwrap())
            {
                bcp_helper(
                    left,
                    &right.left_node.as_ref().unwrap(),
                    r,
                    coreDist,
                    point_set,
                );
                bcp_helper(
                    left,
                    &right.right_node.as_ref().unwrap(),
                    r,
                    coreDist,
                    point_set,
                );
            }
        } else if right.is_leaf() {
            if node_distance(right, &left.left_node.as_ref().unwrap())
                < node_distance(right, &left.right_node.as_ref().unwrap())
            {
                bcp_helper(
                    right,
                    &left.left_node.as_ref().unwrap(),
                    r,
                    coreDist,
                    point_set,
                );
                bcp_helper(
                    right,
                    &left.right_node.as_ref().unwrap(),
                    r,
                    coreDist,
                    point_set,
                );
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
                for i in 0..ordering.len() {
                    bcp_helper(ordering[i].0, ordering[i].1, r, coreDist, point_set);
                }
            }
        }
    }
    r
}

pub fn brute_force_bcp<'a>(
    left: &'a KDTree,
    right: &'a KDTree,
    coreDist: &Vec<f64>,
    point_set: &Vec<Point>,
) -> Bcp<'a> {
    let mut r = Bcp::new();
    for i in 0..left.points.len() {
        for j in 0..right.points.len() {
            let mut dist = f64::max(
                (left.points[i]).distance(&right.points[j]),
                coreDist[point_set.iter().position(|x| x == &left.points[i]).unwrap()],
            );
            dist = f64::max(
                dist,
                coreDist[point_set
                    .iter()
                    .position(|x| x == &right.points[j])
                    .unwrap()],
            );
            r.update(&left.points[i], &right.points[j], dist);
        }
    }
    r
}
