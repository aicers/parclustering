use csv::{self, StringRecord};

use crate::point::Point;

pub fn retrive_points(path: &str) -> Result<Vec<Point>, csv::Error> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b' ')
        .from_path(path);
    //let mut result_vec = Vec::new();
    let mut res_vec: Vec<Point> = Vec::new();

    for result in reader?.records() {
        let rec = result?;

        res_vec.push(Point {
            coords: rec
                .iter()
                .map(|elem| elem.parse::<f32>().expect("Parsing from CSV error"))
                .collect(),
        });
    }

    return Ok(res_vec);
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::sample_points::sample_points;

    use super::*;

    #[test]
    fn get_csv_data() {
        let points_from_csv =
            retrive_points("/home/abdulboriy/Project/kdtree/src/example-data.csv")
                .expect("Could not read from provided path");
        let points_from_rsfile = sample_points();

        assert_eq!(points_from_csv, points_from_rsfile);
        println!("{:#?}", points_from_csv);
    }
}
