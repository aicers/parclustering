#[derive(Debug, Clone)]
pub struct Edge {
    pub u: isize,
    pub v: isize,
    pub directed: bool,
}

impl Default for Edge {
    fn default() -> Self {
        Self {
            u: -1,
            v: -1,
            directed: false,
        }
    }
}

impl Edge {
    pub fn new(u: isize, v: isize, directed: bool) -> Self {
        let (u, v) = if !directed && u > v { (v, u) } else { (u, v) };
        Self { u, v, directed }
    }

    pub fn is_empty(&self) -> bool {
        self.u == -1 && self.v == -1
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.u == other.u && self.v == other.v
    }

    fn ne(&self, other: &Self) -> bool {
        self.u != other.u || self.v != other.v
    }
}

impl Eq for Edge {}

#[derive(Debug, Clone)]
pub struct WeightedEdge {
    pub u: f64,
    pub v: f64,
    pub weight: f64,
    pub directed: bool,
}

impl Default for WeightedEdge {
    fn default() -> Self {
        Self {
            u: -1.0,
            v: -1.0,
            weight: -1.0,
            directed: false,
        }
    }
}
impl WeightedEdge {
    pub fn new(u: f64, v: f64, directed: bool) -> Self {
        let (u, v) = if !directed && u > v { (v, u) } else { (u, v) };
        Self {
            u,
            v,
            weight: -1.0,
            directed,
        }
    }

    pub fn new_weighted(u: f64, v: f64, weight: f64, directed: bool) -> Self {
        let (u, v) = if !directed && u > v { (v, u) } else { (u, v) };
        Self {
            u,
            v,
            weight,
            directed,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.u == -1.0 && self.v == -1.0
    }
}

impl PartialEq for WeightedEdge {
    fn eq(&self, other: &Self) -> bool {
        self.u == other.u && self.v == other.v
    }

    fn ne(&self, other: &Self) -> bool {
        self.u != other.u || self.v != other.v
    }
}

impl Eq for WeightedEdge {}
