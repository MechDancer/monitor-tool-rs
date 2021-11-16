use super::{FigureItem, Vertex};
use iced::{Color, Point};
use std::{
    collections::{hash_map::Entry, vec_deque::Iter, HashMap, VecDeque},
    time::Instant,
};

/// 产生绘图对象的迭代器
pub(super) struct Items<'a> {
    memory: IterMemory,
    iter: Iter<'a, (Instant, Vertex)>,
    color_map: &'a mut HashMap<u8, Color>,
}

/// 迭代绘图缓存
enum IterMemory {
    Vertex(Point, f32, Color),
    Position(Point),
}

macro_rules! get_or_set {
    ($map:expr, $key:expr, $default:expr) => {
        match $map.entry($key) {
            Entry::<_, _>::Occupied(entry) => *entry.get(),
            Entry::<_, _>::Vacant(entry) => *entry.insert($default),
        }
    };
}

impl<'a> Items<'a> {
    pub fn new(
        queue: &'a VecDeque<(Instant, Vertex)>,
        color_map: &'a mut HashMap<u8, Color>,
    ) -> Option<Self> {
        let mut iter = queue.iter();
        iter.next().map(|(_, v)| Items {
            memory: IterMemory::Vertex(v.pos, v.dir, get_or_set!(color_map, v.level, Color::BLACK)),
            iter,
            color_map,
        })
    }
}

impl<'a> Iterator for Items<'a> {
    type Item = FigureItem;

    fn next(&mut self) -> Option<Self::Item> {
        match self.memory {
            IterMemory::Vertex(pos, dir, color) => {
                self.memory = IterMemory::Position(pos);
                if dir.is_nan() {
                    Some(FigureItem::Point(pos, color))
                } else {
                    Some(FigureItem::Arrow(pos, dir, color))
                }
            }
            IterMemory::Position(p0) => self.iter.next().map(|(_, p)| {
                let color = get_or_set!(self.color_map, p.level, Color::BLACK);
                if p.tie {
                    self.memory = IterMemory::Vertex(p.pos, p.dir, color);
                    FigureItem::Tie(p0, p.pos, color)
                } else {
                    self.memory = IterMemory::Position(p.pos);
                    if p.dir.is_nan() {
                        FigureItem::Point(p.pos, color)
                    } else {
                        FigureItem::Arrow(p.pos, p.dir, color)
                    }
                }
            }),
        }
    }
}
