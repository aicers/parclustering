use rand::{thread_rng, Rng};

use crate::point::Point;

pub fn sample_points() -> Vec<Point> {
    let mut points = vec![
        vec![4.64358861061, 3.57238375333, -1.11467509593],
        vec![5.84057255959, 2.98440569083, -2.46964848402],
        vec![9.22158227945, 1.12606946635, 3.50699605829],
        vec![2.27870839345, -1.19956651359, -5.08890628241],
        vec![5.57709629031, -7.75751551455, -1.15129146111],
        vec![-1.33695569845, 4.78896083230, 2.72509287493],
        vec![-7.84699267768, -2.72541126387, 5.17447212222],
        vec![1.97313378995, 1.82374204784, 2.00969580624],
        vec![-2.47732250322, -4.02566042405, 2.42239325508],
        vec![-2.72051015113, 4.39128412694, -1.86318667559],
        vec![4.92752014096, -1.61843701111, 3.32073706377],
        vec![1.29259593339, 3.71582472529, 2.95018887582],
        vec![-1.91910384595, -1.48539948336, 3.30096404704],
        vec![1.35399573838, -5.05795119578, 3.45420508866],
        vec![4.10459998187, -3.85628334112, -1.21347779645],
        vec![-1.33260871817, -4.79089750101, 6.21705278097],
        vec![-6.83344820170, -3.59590332090, -1.67585726658],
        vec![-3.15830743195, 1.34894872536, -1.98958715996],
        vec![-2.27060936089, 4.36058933419, -2.46588902775],
        vec![1.50940651219, 3.17252973510, 4.96312365659],
        vec![2.63167008394, -2.90224504702, -3.95483391158],
        vec![-2.33196681534, -1.34628082351, 5.12677221254],
        vec![-1.42836143716, -1.35883052745, -5.99614126148],
        vec![3.03348248805, 4.99018202759, 2.28326619581],
        vec![2.25073285333, 1.77112588760, 1.74229424796],
        vec![4.14181336606, -3.96008997962, 1.45314857013],
        vec![3.62581642504, -1.95734088232, 5.88999258474],
        vec![-5.45416611812, -2.00464651960, -1.25976648460],
        vec![2.80603623797, -1.43839616654, -3.00449279664],
        vec![-2.60058966560, -2.79817634169, -7.42767449542],
        vec![-4.15496607935, 2.10423527336, -4.74109827060],
        vec![5.40914461841, 1.10103882068, -3.48200823438],
        vec![2.29259686945, -5.17832658743, 8.93490877798],
        vec![-2.17916302691, -4.95377564220, -2.95724174386],
        vec![1.07888570323, 7.58781363415, 3.54978934790],
        vec![-5.62190336903, 2.06750434966, -2.02126731857],
        vec![1.31245618989, 4.54833596382, -1.13854114280],
        vec![-4.42595562425, -1.83816544246, -1.23289962760],
        vec![-2.24800415871, 1.85049250857, 3.65473744260],
        vec![7.14247077585, 2.10709719255, -4.35807421238],
    ];

    let result: Vec<Point> = points
        .iter()
        .map(|i| Point {
            coords: i.iter().map(|i| *i as f32).collect(),
        })
        .collect();
    result
}

pub fn n_random_points(n: usize, d: usize) -> Vec<Point> {
    if n < 1 {
        panic!("Number of Points must be greater than 1");
    } else if d < 1 {
        panic!("Dimension can not be lower than 1");
    }

    let mut rng = thread_rng();
    let n_random = n;

    //Generating random points for our dataset
    let mut make_random_point = || Point {
        coords: (0..d).map(|_| (rng.gen::<f32>() - 0.5) * 100.0).collect(),
    };
    (0..n_random).map(|_| make_random_point()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore = "Checked"]
    #[test]
    fn get_sample_points() {
        let sample_pts = sample_points();

        println!("{sample_pts:?}");
    }

    #[test]
    fn random_pts() {
        let rnd_pts = n_random_points(50, 3);

        println!("{rnd_pts:?}");
    }
}
