use crate::speculative_for::{speculative_for, Reservation};
use crate::{
    hdbscan::WEdge,
    union_find::{EdgeUnionFind, UnionFind},
    wrapper::Wrapper,
};
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

pub trait ReservationFilter {
    fn reserve(&mut self, i: f64) -> bool {
        true
    }
    fn commit(&mut self, i: f64) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
struct IndexedEdge {
    u: f64,
    v: f64,
    id: f64,
    weight: f64,
}

impl IndexedEdge {
    fn new(u: f64, v: f64, id: f64, weight: f64) -> Self {
        Self { u, v, id, weight }
    }
}

#[derive(Debug)]
struct UnionFindStep<'a> {
    e: &'a mut Vec<IndexedEdge>,
    r: &'a mut Vec<Reservation>,
    uf: &'a mut UnionFind,
    is_st: &'a mut Vec<bool>,
}

impl<'a> ReservationFilter for UnionFindStep<'a> {
    fn reserve(&mut self, i: f64) -> bool {
        self.e[i as usize].u = self.uf.find(self.e[i as usize].u);
        let u = self.e[i as usize].u;
        self.e[i as usize].v = self.uf.find(self.e[i as usize].v);
        let v = self.e[i as usize].v;

        if u != v {
            self.r[v as usize].reserve(i);
            self.r[u as usize].reserve(i);
            true
        } else {
            false
        }
    }

    fn commit(&mut self, i: f64) -> bool {
        let u = self.e[i as usize].u;
        let v = self.e[i as usize].v;

        if self.r[v as usize].check(i) {
            self.r[u as usize].check_reset(i);
            self.uf.link(v, u);
            self.is_st[self.e[i as usize].id as usize] = true;
            return true;
        } else if self.r[u as usize].check(i) {
            self.uf.link(u, v);
            self.is_st[self.e[i as usize].id as usize] = true;
            return true;
        } else {
            return false;
        }
    }
}
impl<'a> UnionFindStep<'a> {
    fn new(
        e: &'a mut Vec<IndexedEdge>,
        r: &'a mut Vec<Reservation>,
        uf: &'a mut UnionFind,
        is_st: &'a mut Vec<bool>,
    ) -> Self {
        Self { e, r, uf, is_st }
    }
}

#[derive(Debug)]
struct EdgeUnionFindStep<'a> {
    e: &'a mut Vec<IndexedEdge>,
    r: &'a mut Vec<Reservation>,
    e_real: Vec<IndexedEdge>,
    uf: &'a mut EdgeUnionFind,
}
impl<'a> ReservationFilter for EdgeUnionFindStep<'a> {
    fn reserve(&mut self, i: f64) -> bool {
        self.e[i as usize].u = self.uf.find(self.e[i as usize].u);
        let u = self.e[i as usize].u;
        self.e[i as usize].v = self.uf.find(self.e[i as usize].v);
        let v = self.e[i as usize].v;

        if u != v {
            self.r[v as usize].reserve(i);
            self.r[u as usize].reserve(i);
            true
        } else {
            false
        }
    }

    fn commit(&mut self, i: f64) -> bool {
        let u = self.e[i as usize].u;
        let v = self.e[i as usize].v;
        let u_real = self.e_real[i as usize].u;
        let v_real = self.e_real[i as usize].v;

        if self.r[v as usize].check(i) {
            self.r[u as usize].check_reset(i);
            self.uf
                .link(v, u, v_real, u_real, self.e_real[i as usize].weight);
            return true;
        } else if self.r[u as usize].check(i) {
            self.uf
                .link(u, v, u_real, v_real, self.e_real[i as usize].weight);
            return true;
        } else {
            return false;
        }
    }
}
impl<'a> EdgeUnionFindStep<'a> {
    fn new(
        e: &'a mut Vec<IndexedEdge>,
        r: &'a mut Vec<Reservation>,
        uf: &'a mut EdgeUnionFind,
    ) -> Self {
        let mut e_real = e.clone();
        Self { e, r, e_real, uf }
    }
}

pub fn kruskal(e: &mut Vec<WEdge>, n: usize) -> Vec<f64> {
    let m = e.len();
    let k = std::cmp::min((5 * n) / 4, m);

    let mut iw: Vec<IndexedEdge> = (0..m)
        .map(|i| IndexedEdge::new(e[i].u as f64, e[i].v as f64, i as f64, e[i].weight))
        .collect();

    iw.sort_by(|a, b| {
        if a.weight < b.weight {
            Ordering::Less
        } else if a.weight == b.weight {
            Wrapper(a.id).cmp(&Wrapper(b.id))
        } else {
            Ordering::Greater
        }
    });

    let mut mst_flags = vec![false; m];
    let mut uf = UnionFind::new(n);
    let mut r = vec![Reservation::default(); n];
    let iw_size = iw.len();
    let mut uf_step: Arc<Mutex<UnionFindStep>> = Arc::new(Mutex::new(UnionFindStep::new(
        &mut iw,
        &mut r,
        &mut uf,
        &mut mst_flags,
    )));

    speculative_for(&mut uf_step, 0., iw_size as f64, 20., false, -1.);

    let mst = mst_flags
        .iter()
        .enumerate()
        .filter_map(|(i, &b)| if b { Some(i as f64) } else { None })
        .collect();

    return mst;
}

pub fn batch_kruskal(e: &mut Vec<WEdge>, n: usize, uf: &mut EdgeUnionFind) {
    let m = e.len();
    let k = std::cmp::min((5 * n) / 4, m);

    let mut iw: Vec<IndexedEdge> = (0..m)
        .map(|i| IndexedEdge::new(e[i].u as f64, e[i].v as f64, i as f64, e[i].weight))
        .collect();

    iw.sort_by(|a, b| {
        if a.weight < b.weight {
            Ordering::Less
        } else if a.weight == b.weight {
            Wrapper(a.id).cmp(&Wrapper(b.id))
        } else {
            Ordering::Greater
        }
    });

    let mut r = vec![Reservation::default(); n];
    let iw_size = iw.len();
    let mut uf_step: Arc<Mutex<EdgeUnionFindStep>> =
        Arc::new(Mutex::new(EdgeUnionFindStep::new(&mut iw, &mut r, uf)));
    speculative_for(&uf_step, 0., iw_size as f64, 20., false, -1.);
}
