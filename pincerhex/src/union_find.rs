use crate::tile::Tile;

use std::collections::HashMap;

struct UnionFind {
    set: HashMap<Tile, Element>,
}

#[derive(Copy, Clone)]
struct Element {
    parent: Tile,
    rank: u16,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        Self {
            set: HashMap::with_capacity(size),
        }
    }

    fn clone(&self) -> Self {
        let mut new_set = HashMap::with_capacity(self.set.len());
        for (k, v) in self.set.iter() {
            new_set.insert(*k, *v);
        }
        Self { set: new_set }
    }

    pub fn connected(&mut self, x: Tile, y: Tile) -> bool {
        self.find(x) == self.find(y)
    }

    pub fn find(&mut self, x: Tile) -> Tile {
        if let Some(px) = self.set.get(&x) {
            if px.parent == x {
                return x;
            }
            let gx = self.set[&px.parent];
            if gx.parent == px.parent {
                return px.parent;
            }
            self.set.get_mut(&x).unwrap().parent = gx.parent;
            return self.find(gx.parent);
        }
        self.set.insert(x, Element { parent: x, rank: 0 });
        x
    }

    pub fn union(&mut self, x: Tile, y: Tile) {
        let rep_x = self.find(x);
        let rep_y = self.find(y);
        if rep_x == rep_y {
            return;
        }
        let x = self.set.get(&rep_x).unwrap();
        let y = self.set.get(&rep_y).unwrap();

        if x.rank < y.rank {
            self.set.get_mut(&rep_x).unwrap().parent = y.parent;
        } else if x.rank > y.rank {
            self.set.get_mut(&rep_y).unwrap().parent = x.parent;
        } else {
            self.set.get_mut(&rep_x).unwrap().parent = y.parent;
            self.set.get_mut(&rep_y).unwrap().rank += 1;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_union() {
        let mut uf = UnionFind::new(0);
        assert!(!uf.connected(Tile::Edge1, Tile::Edge2));
        uf.union(Tile::Edge1, Tile::Edge2);
        assert!(uf.connected(Tile::Edge1, Tile::Edge2));
    }
}
