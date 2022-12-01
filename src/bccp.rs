/* Fast Parallel Algorithm: Computation of Bichromatic Closest Pair Problem Source Code
Implemented:
1. Computing BCCP(original version) using Euclidean distance metric for n-dimensional points

2.Pending Implementation:
A. Need to resolve reference issue on assignning values to euclidean_distance function
now it is working with .copy() only, otherwise panicing.
B. Adding the cases when one of the sets might be empty.
C. Calculation of BCCP using core distances*/

#[derive(Debug, Clone, Copy)]
pub struct Bccp<T> {
    left_node: Option<T>,
    right_node: Option<T>,
}

impl<T> Bccp<T>
where
    T: Iterator<Item = Vec<f64>> + Copy + PartialEq,
{
    #[must_use]
    pub fn new(left_node: Option<T>, right_node: Option<T>) -> Self {
        Self {
            left_node,
            right_node,
        }
    }

    pub fn calculate_distance(&self) -> ((Vec<f64>, Vec<f64>), f64) {
        let (mut closest_pairs, mut pair_distance) = ((Vec::new(), Vec::new()), 0.);
        // temp_var was added just to make sure that we are in index 0
        let mut temp_var: bool = true;

        if Some(self.left_node) != None && Some(self.right_node) != None {
            for left_point in self.left_node.unwrap() {
                for right_point in self.right_node.unwrap() {
                    let euclidean_dist = euclidean_distance(&left_point, &right_point);
                    if temp_var {
                        pair_distance = euclidean_dist;
                        (closest_pairs.0, closest_pairs.1) =
                            (left_point.clone(), right_point.clone());
                        temp_var = false;
                    } else if euclidean_dist < pair_distance {
                        pair_distance = euclidean_dist;
                        (closest_pairs.0, closest_pairs.1) =
                            (left_point.clone(), right_point.clone());
                    }
                }
            }
        } else if Some(self.left_node) != None {
        } else if Some(self.right_node) != None {
        } else {
            panic!("Both sets are empty")
        }
        (closest_pairs, pair_distance)
    }
}

fn euclidean_distance(left_point: &[f64], right_point: &[f64]) -> f64 {
    left_point
        .iter()
        .zip(right_point.iter())
        .fold(0., |mut sum, (&x1, &x2)| {
            let diff = x1 - x2;
            sum += diff * diff;
            sum
        })
        .sqrt()
}
#[cfg(test)]
mod tests {
    use crate::Bccp;
    #[test]
    fn calculation_check() {
        let test_data1: Option<_> = Some([[1., 4., 3., 4.], [1., 4., 3., 4.], [1., 4., 3., 4.]]);

        let test_data2: Option<_> = Some([[5., 7., 3., 1.], [1., 4., 3., 4.], [1., 4., 3., 4.]]);

        let minimum_distance = Bccp {
            left_node: test_data1.as_ref(),
            right_node: test_data2.as_ref(),
        };
        println!("{:?}", minimum_distance);
    }
}
