
use std::hash::Hash;
use std::ops::Range;
use std::collections::{hash_set, HashMap, HashSet};

const HASH_X: i32 = 1888347217;
const HASH_Y: i32 = 1424909897;
const BIN_SIZE: i32 = 8;
const GRID_COUNT: usize = 1024;

pub struct Grid<T> {
    eids: HashMap<T, Rectangle>,
    bins: Vec<HashSet<T>>
}

fn bin(x: i32, y: i32) -> usize {
    let coord = x.wrapping_mul(HASH_X).wrapping_add(y.wrapping_mul(HASH_Y));
    coord as usize % GRID_COUNT
}

#[derive(Debug, Copy, Clone)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32
}

impl Rectangle {
    fn iter(&self) -> RecIter {
        let range_x = (self.x/BIN_SIZE)..((self.x+self.width)/BIN_SIZE+1);
        let range_y = (self.y/BIN_SIZE)..((self.y+self.height)/BIN_SIZE+1);

        RecIter {
            cx: 0..0,
            cy: 0,
            x: range_x,
            y: range_y
        }
    }

    fn max(&self) -> (i32, i32) {
        (self.x + self.width, self.y + self.height)
    }

    fn intersects(&self, b: Rectangle) -> bool {
        let a = self;
        let (a_max_x, a_max_y) = a.max();
        let (b_max_x, b_max_y) = b.max();

        !(a.x >= b_max_x || a.y >= b_max_y || b.x >= a_max_x || b.y >= a_max_y)
    }
}

#[derive(Debug)]
pub struct RecIter {
    cx: Range<i32>,
    cy: i32,
    x: Range<i32>,
    y: Range<i32>
}

impl Iterator for RecIter {
    type Item = (i32, i32);
    fn next(&mut self) -> Option<(i32, i32)> {
        loop {
            if let Some(x) = self.cx.next() {
                return Some((x, self.cy));
            }
            if let Some(y) = self.y.next() {
                self.cy = y;
                self.cx = self.x.clone();
            } else {
                return None;
            }
        }
    }
}

impl<T: Copy+Eq+Hash> Grid<T> {
    /// Create an empty grid
    pub fn new() -> Grid<T> {
        Grid {
            eids: HashMap::new(),
            bins: (0..GRID_COUNT).map(|_| HashSet::new()).collect()
        }
    }

    /// insert a rectangle into the grid
    pub fn insert(&mut self, rec: Rectangle, eid: T) {
        for (x, y) in rec.iter() {
            self.bins[bin(x, y)].insert(eid);
        }
        self.eids.insert(eid, rec);
    }

    ///
    pub fn intersects(&self, rec: Rectangle) -> Iter<T> {
        Iter {
            grid: self,
            iter: rec.iter(),
            bin: None,
            rec: rec
        }
    }
}

pub struct Iter<'a, T:'a> {
    grid: &'a Grid<T>,
    rec: Rectangle,
    bin: Option<hash_set::Iter<'a, T>>,
    iter: RecIter
}

impl<'a, T:Copy+Eq+Hash> Iterator for Iter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        loop {
            if let Some(ref mut bin) = self.bin {
                while let Some(e) = bin.next() {
                    if self.grid.eids[e].intersects(self.rec) {
                        return Some(*e);
                    }
                }
            }

            if let Some((x, y)) = self.iter.next() {
                let b = bin(x, y);
                self.bin = Some(self.grid.bins[b].iter());
            } else {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use grid::{Grid, Rectangle};

    #[test]
    fn intersects() {
        let mut grid = Grid::new();

        let mut i = 0..;
        for x in -10..10 {
            for y in -10..10 {
                grid.insert(Rectangle{
                    x: x*2, y: y*2,
                    width: 2, height: 2
                }, i.next().unwrap());
            }
        }

        for x in -10..10 {
            for y in -10..10 {
                assert!(grid.intersects(Rectangle{
                    x: x*2, y: y*2,
                    width: 2, height: 2
                }).next().is_some());
            }
        }

    }

}