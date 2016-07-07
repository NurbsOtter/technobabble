
use std;

const FANOUT_SIZE: usize = 8;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MortonPoint(pub u32);

fn interleave(x: u32) -> u32 {
    let x = (x | (x << 8)) & 0x00ff_00ff;
    let x = (x | (x << 4)) & 0x0f0f_0f0f;
    let x = (x | (x << 2)) & 0x3333_3333;
    (x | (x << 1)) & 0x5555_5555
}

fn deinterleave(x: u32) -> u32 {
    let x = x & 0x5555_5555;
    let x = (x | (x >> 1)) & 0x3333_3333;
    let x = (x | (x >> 2)) & 0x0f0f_0f0f;
    let x = (x | (x >> 4)) & 0x00ff_00ff;
    (x | (x >> 8)) & 0x0000_ffff
}

impl Point {
    /// Create a new point
    pub fn new(x: i16, y: i16) -> Point {
        Point { x: x, y: y }
    }
}

impl From<Point> for MortonPoint {
    fn from(p: Point) -> MortonPoint {
        let x = (p.x as u32).wrapping_sub(i16::min_value() as u32);
        let y = (p.y as u32).wrapping_sub(i16::min_value() as u32);
        MortonPoint(interleave(x) | interleave(y) << 1)
    }
}

impl From<MortonPoint> for Point {
    fn from(p: MortonPoint) -> Point {
        let x = deinterleave(p.0).wrapping_add(i16::min_value() as u32);
        let y = deinterleave(p.0 >> 1).wrapping_add(i16::min_value() as u32);
        Point {
            x: x as i16,
            y: y as i16,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rectangle {
    pub min: Point,
    pub max: Point,
}

impl Rectangle {
    /// does anypoint in the rectangle touch the lhs.
    /// this includes the case where two rectangles share an edge
    pub fn intersects(&self, b: Rectangle) -> bool {
        let a = self;
        !(a.min.x > b.max.x || a.min.y > b.max.y || b.min.x > a.max.x || b.min.y > a.max.y)
    }

    /// check to see if lhs overlaops with b. This does not include
    /// the case where they share an edge
    pub fn overlaps(&self, b: Rectangle) -> bool {
        let a = self;
        !(a.min.x >= b.max.x || a.min.y >= b.max.y || b.min.x >= a.max.x || b.min.y >= a.max.y)
    }

    pub fn extend(self, b: Rectangle) -> Rectangle {
        let a = self;
        Rectangle {
            min: Point {
                x: std::cmp::min(a.min.x, b.min.x),
                y: std::cmp::min(a.min.y, b.min.y),
            },
            max: Point {
                x: std::cmp::max(a.max.x, b.max.x),
                y: std::cmp::max(a.max.y, b.max.y),
            },
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MortonRectangle {
    pub min: MortonPoint,
    pub max: MortonPoint,
}

impl MortonRectangle {
    fn center(&self) -> u32 {
        (self.min.0 >> 1) + (self.max.0 >> 1)
    }
}

impl From<Rectangle> for MortonRectangle {
    fn from(p: Rectangle) -> MortonRectangle {
        MortonRectangle {
            min: p.min.into(),
            max: p.max.into(),
        }
    }
}

impl From<MortonRectangle> for Rectangle {
    fn from(p: MortonRectangle) -> Rectangle {
        Rectangle {
            min: p.min.into(),
            max: p.max.into(),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PointsAt {
    Leaf,
    Inner,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node {
    rect: Rectangle,
    leaf: PointsAt,
    start: usize,
    stop: usize,
}

pub struct RTree<T> {
    nodes: Vec<Node>,
    values: Vec<(Rectangle, T)>,
}

impl<T> RTree<T> {
    pub fn new() -> RTree<T> {
        RTree {
            nodes: vec![],
            values: vec![],
        }
    }

    fn rebuild(&mut self) {
        self.values.sort_by(|a, b| {
            let a: MortonRectangle = a.0.into();
            let b: MortonRectangle = b.0.into();
            a.center().cmp(&b.center())
        });
        self.nodes.clear();

        let (mut start, stop) = (0, self.values.len());
        while start != stop {
            let end = std::cmp::min(stop, start + FANOUT_SIZE);
            let mut rect = self.values[start].0;
            for i in (start + 1)..end {
                rect = rect.extend(self.values[i].0);
            }
            self.nodes.push(Node {
                rect: rect,
                leaf: PointsAt::Leaf,
                start: start,
                stop: end,
            });
            start = end;
        }

        let (mut start, mut stop) = (0, self.nodes.len());
        while stop - start > 1 {
            while start != stop {
                let end = std::cmp::min(stop, start + FANOUT_SIZE);
                let mut rect = self.nodes[start].rect;
                for i in (start + 1)..end {
                    rect = rect.extend(self.nodes[i].rect);
                }
                self.nodes.push(Node {
                    rect: rect,
                    leaf: PointsAt::Inner,
                    start: start,
                    stop: end,
                });
                start = end;
            }
            start = stop;
            stop = self.nodes.len();
        }
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.nodes.clear();
    }

    pub fn extend<I>(&mut self, iter: I)
        where I: IntoIterator<Item = (Rectangle, T)>
    {
        self.values.extend(iter);
        self.rebuild();
    }

    pub fn query(&self, rect: Rectangle) -> Iter<T> {
        let mut to_check = vec![];

        if self.nodes.len() >= 1 {
            let last = self.nodes.len() - 1;
            if self.nodes[last].rect.intersects(rect) {
                to_check.push(self.nodes[last])
            }
        }

        Iter {
            query: rect,
            tree: self,
            to_check: to_check,
        }
    }
}

impl<A> std::iter::FromIterator<(Rectangle, A)> for RTree<A> {
    fn from_iter<T>(iter: T) -> Self
        where T: IntoIterator<Item = (Rectangle, A)>
    {
        let mut rtree = RTree::new();
        rtree.extend(iter);
        rtree
    }
}

fn inc(v: &mut usize) -> usize {
    let old = *v;
    *v += 1;
    old
}

pub struct Iter<'a, T: 'a> {
    query: Rectangle,
    tree: &'a RTree<T>,
    to_check: Vec<Node>,
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = (&'a Rectangle, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            if let Some(mut last) = self.to_check.pop() {
                let ret = if last.leaf == PointsAt::Leaf {
                    let mut ret = None;
                    while ret.is_none() && last.start != last.stop {
                        let idx = inc(&mut last.start);
                        if self.tree.values[idx].0.intersects(self.query) {
                            ret = Some((&self.tree.values[idx].0, &self.tree.values[idx].1));
                        }
                    }

                    if last.start != last.stop {
                        self.to_check.push(last);
                    }
                    ret
                } else {
                    let mut idx;
                    loop {
                        idx = inc(&mut last.start);
                        if idx == last.stop {
                            continue 'outer;
                        }
                        if self.tree.nodes[idx].rect.intersects(self.query) {
                            break;
                        }
                    }

                    self.to_check.push(last);
                    self.to_check.push(self.tree.nodes[idx]);
                    None
                };

                if ret.is_some() {
                    return ret;
                }
            } else {
                return None;
            };
        }
    }
}


#[cfg(test)]
mod test {
    use std::i16;
    use {Rectangle, Point, MortonPoint, RTree};

    #[test]
    fn point() {
        let pt = Point::new(0, 0);
        assert_eq!(MortonPoint(0xC000_0000), MortonPoint::from(pt));

        let pt = Point::new(i16::min_value(), i16::min_value());
        assert_eq!(MortonPoint(0x0000_0000), MortonPoint::from(pt));

        let pt = Point::new(i16::max_value(), i16::max_value());
        assert_eq!(MortonPoint(0xFFFF_FFFF), MortonPoint::from(pt));

        let pt = Point::new(i16::max_value(), i16::min_value());
        assert_eq!(MortonPoint(0x5555_5555), MortonPoint::from(pt));

        let pt = Point::new(i16::min_value(), i16::max_value());
        assert_eq!(MortonPoint(0xAAAA_AAAA), MortonPoint::from(pt));
    }

    #[test]
    fn encode() {
        for i in i16::min_value()..i16::max_value() {
            let src = Point::new(i, 0);
            let out = Point::from(MortonPoint::from(src));
            assert_eq!(src, out);
        }

        for i in i16::min_value()..i16::max_value() {
            let src = Point::new(0, i);
            let out = Point::from(MortonPoint::from(src));
            assert_eq!(src, out);
        }
    }


    #[test]
    fn insert() {
        let mut keys = Vec::new();

        for x in -10..10 {
            for y in -10..10 {
                let rect = Rectangle {
                    min: Point::new(x * 10, y * 10),
                    max: Point::new(x * 10 + 5, y * 10 + 5),
                };
                keys.push((rect, (x, y)));
            }
        }

        let tree: RTree<_> = keys.drain(..).collect();

        assert_eq!(400,
                   tree.query(Rectangle {
                           max: Point { x: 100, y: 100 },
                           min: Point { x: -100, y: -100 },
                       })
                       .count());

        for x in -10..10 {
            for y in -10..10 {
                assert_eq!(1,
                           tree.query(Rectangle {
                                   min: Point::new(x * 10, y * 10),
                                   max: Point::new(x * 10 + 5, y * 10 + 5),
                               })
                               .count());
            }
        }

        for x in -5..5 {
            for y in -5..5 {
                assert_eq!(4,
                           tree.query(Rectangle {
                                   min: Point::new(x * 20, y * 20),
                                   max: Point::new(x * 20 + 15, y * 20 + 15),
                               })
                               .count());
            }
        }

        for x in -2..2 {
            for y in -2..2 {
                assert_eq!(25,
                           tree.query(Rectangle {
                                   min: Point::new(x * 50, y * 50),
                                   max: Point::new(x * 50 + 45, y * 50 + 45),
                               })
                               .count());
            }
        }

        for x in -1..1 {
            for y in -1..1 {
                assert_eq!(100,
                           tree.query(Rectangle {
                                   min: Point::new(x * 100, y * 100),
                                   max: Point::new(x * 100 + 95, y * 100 + 95),
                               })
                               .count());
            }
        }
    }

    #[test]
    fn insert_close() {
        let mut keys = Vec::new();

        for x in -100..100 {
            for y in -100..100 {
                let rect = Rectangle {
                    min: Point::new(x, y),
                    max: Point::new(x, y),
                };
                keys.push((rect, (x, y)));
            }
        }

        let tree: RTree<_> = keys.drain(..).collect();

        let all = Rectangle {
            min: Point::new(-100, -100),
            max: Point::new(100, 100),
        };

        assert_eq!(tree.query(all.into()).count(), 40_000);
    }
}