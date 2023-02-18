use csv;

use crate::point::Point;

pub fn retrive_points(path: &str) -> Result<Vec<Point>, csv::Error> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut result_vec: Vec<Point> = Vec::new();
    for result in reader.records() {
        result_vec.push(Point {
            coords: result
                .iter()
                .flat_map(|i| i.iter().map(|j| j.parse::<f32>().unwrap()))
                .collect(),
        });
    }
    return Ok(result_vec);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_csv_data() {
        let points =
            retrive_points("/home/abdulboriy/Project/kdtree/src/example-data.csv").unwrap();
        println!("{points:?}");
    }
}
