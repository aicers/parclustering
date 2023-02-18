use crate::union_find::UFConstruct;
use std::sync::{Arc, Mutex};

use crate::{
    kdtree::KDTree,
    point::Point,
    union_find::{EdgeUnionFind, UnionFind},
};

pub fn mark_all(tree: &mut KDTree, n: i64) {
    if !tree.is_leaf() && tree.get_id() != n {
        if tree.size() > 2000 {
            rayon::join(
                || {
                    if let Some(ref mut left_node) = tree.left_node {
                        mark_all(left_node, n);
                    }
                },
                || {
                    if let Some(ref mut right_node) = tree.right_node {
                        mark_all(right_node, n);
                    }
                },
            );
        } else {
            if let Some(ref mut left_node) = tree.left_node {
                mark_all(left_node, n);
            }

            if let Some(ref mut right_node) = tree.right_node {
                mark_all(right_node, n);
            }
        }
    }
    tree.set_id(n);
}

pub fn mark(node: &mut KDTree, uf: &mut Arc<Mutex<EdgeUnionFind>>, s: &Vec<Point>) {
    if node.has_id() {
        mark_all(
            node,
            uf.lock()
                .unwrap()
                .find(s.iter().position(|x| *x == node.points[0]).unwrap() as i64),
        );
        return;
    }

    node.set_id(
        uf.lock()
            .unwrap()
            .find(s.iter().position(|x| *x == node.points[0]).unwrap() as i64),
    );

    if node.is_leaf() {
        for i in 1..node.size() {
            if node.get_id()
                != uf
                    .lock()
                    .unwrap()
                    .find(s.iter().position(|x| *x == node.points[i]).unwrap() as i64)
            {
                node.reset_id();
                return;
            }
        }
    } else {
        if node.size() > 2000 {
            rayon::join(
                || {
                    if let Some(ref mut left_node) = node.left_node {
                        mark(left_node, &mut uf.clone(), s)
                    }
                },
                || {
                    if let Some(ref mut right_node) = node.right_node {
                        mark(right_node, &mut uf.clone(), s)
                    }
                },
            );
        } else {
            if let Some(ref mut left_node) = node.left_node {
                mark(left_node, uf, s)
            }

            if let Some(ref mut right_node) = node.right_node {
                mark(right_node, uf, s)
            }
        }

        if node.get_id() != node.left_node.as_ref().unwrap().get_id() {
            node.reset_id();
            return;
        }

        if node.get_id() != node.right_node.as_ref().unwrap().get_id() {
            node.reset_id();
            return;
        }
    }
}
