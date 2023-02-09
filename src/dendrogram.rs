use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::edge::WeightedEdge;
use crate::union_find::{UFConstruct, UnionFind};
use crate::wrapper::Wrapper;

#[derive(Debug, Clone)]
pub struct DendroNode {
    left: usize,
    right: usize,
    weight: f64,
    size: usize,
}

impl Default for DendroNode {
    fn default() -> Self {
        Self {
            left: 0,
            right: 0,
            weight: 0.,
            size: 0,
        }
    }
}
impl DendroNode {
    pub fn new(left: usize, right: usize, weight: f64, size: usize) -> Self {
        Self {
            left,
            right,
            weight,
            size,
        }
    }
}

pub fn dendrogram(edges: Vec<WeightedEdge>, n: usize) -> Vec<DendroNode> {
    let mut edges = edges.clone();
    edges.sort_by(|a, b| Wrapper(a.weight).cmp(&Wrapper(b.weight)));

    let mut uf = UnionFind::new(n);

    let mut idx = n;
    let mut idx_map = vec![0; n];
    let mut sizes = vec![0; n];

    (for i in 0..n {
        idx_map[i] = i;
        sizes[i] = 1;
    });

    let mut dendro = vec![DendroNode::default(); edges.len()];

    for i in 0..n - 1 {
        let u = uf.find(edges[i].u);
        let v = uf.find(edges[i].v);

        dendro[i] = DendroNode::new(
            u as usize,
            v as usize,
            edges[i].weight,
            sizes[u as usize] + sizes[v as usize],
        );
        uf.link(u, v);
        let new_idx = uf.find(u);
        idx_map[new_idx as usize] = idx;
        sizes[new_idx as usize] = sizes[u as usize] + sizes[v as usize];
        idx += 1;
    }

    dendro
}
