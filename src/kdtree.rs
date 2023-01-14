use crate::point::Point;
use crate::quickselect::quickselect_by;
use crate::wrapper::Wrapper;

use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::ContinuousView;
use rand::prelude::*;
use std::cmp::{Ord, Ordering};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct KDTree {
    pub id: i32,
    pub points: Vec<Point>,
    pub split_value: Point,
    pub left_node: Option<Box<KDTree>>,
    pub right_node: Option<Box<KDTree>>,
    pub dimension: usize,
    pub cd_min: f64,
    pub cd_max: f64,
}

impl Ord for KDTree {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut res = Ordering::Equal;
        for (x, y) in self
            .split_value
            .coords
            .iter()
            .zip(other.split_value.coords.iter())
        {
            res = Wrapper(*x).cmp(&Wrapper(*y));
            if res != Ordering::Equal {
                break;
            }
        }
        res
    }
}

impl PartialOrd for KDTree {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut res = Ordering::Equal;
        for (x, y) in self
            .split_value
            .coords
            .iter()
            .zip(other.split_value.coords.iter())
        {
            res = Wrapper(*x).cmp(&Wrapper(*y));
            if res != Ordering::Equal {
                break;
            }
        }
        Some(res)
    }
}

impl PartialEq for KDTree {
    fn eq(&self, other: &Self) -> bool {
        self.split_value.coords == other.split_value.coords
    }
}

impl Eq for KDTree {}

impl KDTree {
    pub fn build(point_list: &mut Vec<Point>) -> KDTree {
        KDTree::new(point_list, 0)
    }

    pub fn empty() -> Self {
        Self {
            id: -1,
            points: vec![Point::default()],
            split_value: Point::default(),
            left_node: None,
            right_node: None,
            dimension: 0,
            cd_min: 0.,
            cd_max: 0.,
        }
    }
    pub fn new(point_list: &mut Vec<Point>, dim: usize) -> KDTree {
        let points_len = point_list.len();
        if points_len == 1 {
            return KDTree {
                id: -1,
                points: point_list.to_vec(),
                split_value: point_list[0].clone(),
                left_node: None,
                right_node: None,
                dimension: dim,
                cd_min: 0.,
                cd_max: 0.,
            };
        } else if point_list.is_empty() {
            return KDTree {
                id: -1,
                points: point_list.to_vec(),
                split_value: Point { coords: vec![0.] },
                left_node: None,
                right_node: None,
                dimension: 0,
                cd_min: 0.,
                cd_max: 0.,
            };
        }

        let pivot = quickselect_by(point_list, points_len / 2, &|a, b| {
            a.coords[dim].partial_cmp(&b.coords[dim]).unwrap()
        });
        let left_node = if point_list.len() >= 1 {
            Some(Box::new(KDTree::new(
                &mut point_list[0..points_len / 2].to_vec(),
                (dim + 1) % pivot.coords.len(),
            )))
        } else {
            None
        };

        let right_node = if point_list.len() >= 2 {
            Some(Box::new(KDTree::new(
                &mut point_list[points_len / 2..points_len].to_vec(),
                (dim + 1) % pivot.coords.len(),
            )))
        } else {
            None
        };

        KDTree {
            id: -1,
            points: point_list.to_vec(),
            split_value: pivot,
            dimension: dim,
            left_node: left_node,
            right_node: right_node,
            cd_min: 0.,
            cd_max: 0.,
        }
    }
    pub fn is_leaf(&self) -> bool {
        if Some(&self.left_node) == None && Some(&self.right_node) == None {
            true
        } else {
            false
        }
    }
    pub fn reset_id(&mut self) {
        self.id = -1;
    }

    pub fn set_id(&mut self, n: i32) {
        self.id = n;
    }

    pub fn has_id(&self) -> bool {
        self.id != -1
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_min(&self, index_dim: usize) -> f64 {
        let mut points = self.points.clone();
        let max_elem_index = 1;
        let res = quickselect_by(&mut points, max_elem_index, &|a, b| {
            a.coords[index_dim]
                .partial_cmp(&b.coords[index_dim])
                .unwrap()
        });
        res.coords[index_dim]
    }

    pub fn size(&self) -> usize {
        self.points.len()
    }
    pub fn get_max(&self, index_dim: usize) -> f64 {
        let mut points = self.points.clone();
        let max_elem_index = points.len();
        let res = quickselect_by(&mut points, max_elem_index, &|a, b| {
            a.coords[index_dim]
                .partial_cmp(&b.coords[index_dim])
                .unwrap()
        });
        res.coords[index_dim]
    }

    pub fn lMax(&self) -> f64 {
        let mut max_val: f64 = 0.0;
        let point_dim = self.points[0].coords.len();
        for d in 0..point_dim {
            let temp_val = self.get_max(d) - self.get_min(d);
            if temp_val > max_val {
                max_val = temp_val;
            }
        }
        max_val
    }

    pub fn diag(&self) -> f64 {
        let mut res = 0.;

        for d in 0..self.size() {
            let tmp = self.get_max(d) - self.get_min(d);
            res += tmp * tmp;
        }
        return f64::sqrt(res);
    }

    pub fn nearest_neighbours(&self, point: &Point, k: usize) -> Vec<(Wrapper, Point)> {
        let mut queue = VecDeque::new();
        let mut closest_points: Vec<(Wrapper, Point)> = Vec::new();

        queue.push_back(self);

        while !queue.is_empty() {
            let node = queue.pop_front().unwrap();

            if node.points.len() == 1 {
                let distance = Wrapper(point.distance(&node.points[0]));
                if closest_points.len() >= k {
                    closest_points.sort_by(|a, b| a.0.cmp(&b.0));
                    let max_distance: Wrapper = closest_points.last().unwrap().0;
                    if distance < max_distance {
                        closest_points.pop();
                        if !closest_points.contains(&(distance, node.points[0].clone())) {
                            closest_points.push((distance, node.points[0].clone()));
                        }
                    }
                } else {
                    if !closest_points.contains(&(distance, node.points[0].clone())) {
                        closest_points.push((distance, node.points[0].clone()));
                    }
                }
            } else {
                if let Some(ref left_node) = node.left_node {
                    queue.push_back(left_node.as_ref());
                }
                if let Some(ref right_node) = node.right_node {
                    queue.push_back(right_node.as_ref());
                }
            }
        }
        closest_points
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn bigger_input() {
        let mut rng = thread_rng();
        let n_random = 1_000_000;
        let mut make_random_point = || Point {
            coords: (0..100)
                .map(|_| (rng.gen::<f64>() - 0.5) * 1000000.0)
                .collect(),
        };
        let mut random_points: Vec<Point> = (0..n_random).map(|_| make_random_point()).collect();
        let search_point = random_points[0].clone();
        let mut copy = random_points.clone();

        let kdtree = KDTree::build(&mut random_points);
        let closest_pts = kdtree.nearest_neighbours(&search_point, 4);

        copy.sort_by_key(|point| Wrapper(point.distance(&search_point)));

        assert_eq!(closest_pts[0].1, copy[0]);
        assert_eq!(closest_pts[1].1, copy[1]);
        assert_eq!(closest_pts[2].1, copy[2]);
        assert_eq!(closest_pts[3].1, copy[3]);
    }

    #[test]
    fn std_test() {
        let mut rng = thread_rng();
        let mut wp_points: Vec<Point> = (0..5)
            .map(|_| Point {
                coords: vec![rng.gen_range(0.0..30.0), rng.gen_range(0.0..30.0)],
            })
            .collect();
        println!("{:?}", wp_points);
        let kdtree = KDTree::build(&mut wp_points);
        let target = Point {
            coords: vec![10., 8.],
        };
        let closest_points = kdtree.nearest_neighbours(&target, 3);
        //println!("{:?}", closest_points);
        println!("{:?}", wp_points);
        let closest_points: Vec<(f64, f64)> = closest_points
            .iter()
            .map(|x| (x.1.coords[0], x.1.coords[1]))
            .collect();
        let wp_points = wp_points
            .iter()
            .filter(|x| !closest_points.contains(&(x.coords[0], x.coords[1])))
            .map(|x| (x.coords[0], x.coords[1]))
            .collect();

        let s1 = Plot::new(vec![(10., 8.)])
            .point_style(PointStyle::new().marker(PointMarker::Square).colour("red"));

        let s2 = Plot::new(closest_points)
            .point_style(PointStyle::new().marker(PointMarker::Square).colour("blue"));

        let s3 = Plot::new(wp_points).point_style(
            PointStyle::new()
                .marker(PointMarker::Square)
                .colour("black"),
        );

        let v = ContinuousView::new().add(s1).add(s2).add(s3);

        plotlib::page::Page::single(&v).save("kdtree.svg").unwrap();
    }
    /*
    #[test]
    fn empty_list() {
        let mut empty_vec = vec![Point { coords: vec![0.0] }];

        let kdtree = KDTree::build(&mut empty_vec);

        println!("{:#?}", kdtree);
    }

    #[test]
    fn one_point() {
        let mut single_val = vec![Point { coords: vec![0.] }];

        let kdtree = KDTree::build(&mut single_val);

        println!("{:#?}", kdtree);
    }

    #[test]
    fn find_nearest_points() {
        let mut wp_points: Vec<Point> = [
            [2.0, 3.0],
            [5.0, 4.0],
            [9.0, 6.0],
            [4.0, 7.0],
            [8.0, 1.0],
            [7.0, 2.0],
        ]
        .iter()
        .map(|x| Point { coords: x.to_vec() })
        .collect();

        let kdtree = KDTree::build(&mut wp_points);
        let target: Point = Point {
            coords: vec![2., 3.],
        };
        let nearest = kdtree.nearest_neighbours(&target, 3);

        println!("{:?}", nearest);
        println!("{:#?}", kdtree);
    }*/
}
