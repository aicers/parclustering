use crate::bccp::{self, Bcp};
use crate::edge::WeightedEdge;

pub trait UFConstruct {
    fn is_root(&self, i: f64) -> bool {
        true
    }
    fn find(&mut self, i: f64) -> f64 {
        0.
    }
}

#[derive(Debug)]
pub struct UnionFind {
    parents: Vec<f64>,
}
impl UFConstruct for UnionFind {
    fn is_root(&self, u: f64) -> bool {
        return self.parents[u as usize] == -1.0;
    }

    fn find(&mut self, i: f64) -> f64 {
        let mut i = i;
        if self.is_root(i) {
            return i as f64;
        }
        let mut p = self.parents[i as usize];

        if self.is_root(p) {
            return p as f64;
        }
        while !self.is_root(p) {
            let gp = self.parents[p as usize];
            self.parents[i as usize] = gp;
            i = p;
            p = (gp as usize) as f64;
        }
        return p as f64;
    }
}

impl UnionFind {
    pub fn new(n: usize) -> Self {
        Self {
            parents: vec![-1.0; n],
        }
    }
    fn union_roots(&mut self, u: f64, v: f64) {
        let (u, v) = if self.parents[v as usize] < self.parents[u as usize] {
            (v, u)
        } else {
            (u, v)
        };

        self.parents[u as usize] += self.parents[v as usize];
        self.parents[v as usize] = u;
    }

    pub fn link(&mut self, u: f64, v: f64) {
        self.parents[u as usize] = v as f64;
    }

    /*fn try_link(&mut self, u: usize, v: usize) -> bool {
        return self.parents[u];
    }*/
}

type WghEdge = WeightedEdge;
#[derive(Debug)]
pub struct EdgeUnionFind {
    parents: Vec<f64>,
    edges: Vec<WghEdge>,
}

impl UFConstruct for EdgeUnionFind {
    fn is_root(&self, u: f64) -> bool {
        self.parents[u as usize] == -1.0
    }

    fn find(&mut self, i: f64) -> f64 {
        let mut i = i;
        if self.is_root(i) {
            return i;
        }
        let mut p = self.parents[i as usize];
        if self.is_root(p) {
            return p;
        }

        while !self.is_root(p) {
            let gp = self.parents[p as usize];
            self.parents[i as usize] = gp;
            i = p;
            p = gp;
        }
        return p;
    }
}

impl EdgeUnionFind {
    pub fn new(n: usize) -> Self {
        Self {
            parents: vec![-1.0; n],
            edges: vec![WeightedEdge::default(); n],
        }
    }

    pub fn link(&mut self, u: f64, v: f64, u_real: f64, v_real: f64, weight: f64) {
        self.edges[u as usize] = WghEdge::new_weighted(u_real, v_real, weight, false);
        self.parents[u as usize] = v;
    }

    pub fn num_edge(&self) -> usize {
        self.edges.iter().filter(|e| !e.is_empty()).count()
    }

    pub fn get_edge(&self) -> Vec<WghEdge> {
        self.edges
            .iter()
            .filter(|e| !e.is_empty())
            .cloned()
            .collect()
    }
}
