use crate::figure::aabb;

use super::{FigureItem, Vertex, AABB};
use iced::{Color, Point};
use std::{
    collections::{vec_deque::Iter, HashMap, VecDeque},
    time::Instant,
};

/// 产生绘图对象的迭代器
pub(super) struct Items<'a> {
    memory: IterMemory,
    aabb: AABB,
    iter: Iter<'a, (Instant, Vertex)>,
    color_map: &'a mut HashMap<u8, Color>,
}

/// 迭代绘图缓存
enum IterMemory {
    Vertex(Point, f32, Color),
    Position(Point),
}

impl<'a> Items<'a> {
    pub fn new(
        queue: &'a VecDeque<(Instant, Vertex)>,
        color_map: &'a mut HashMap<u8, Color>,
        aabb: AABB,
    ) -> Option<Self> {
        let mut iter = queue.iter();
        iter.next().map(|(_, v)| Items {
            memory: IterMemory::Vertex(
                Point { x: v.x, y: v.y },
                v.dir,
                *color_map.entry(v.level).or_insert(Color::BLACK),
            ),
            aabb,
            iter,
            color_map,
        })
    }
}

impl<'a> Iterator for Items<'a> {
    type Item = FigureItem;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.memory {
                IterMemory::Vertex(pos, dir, color) => {
                    self.memory = IterMemory::Position(pos);
                    if self.aabb.contains(pos) {
                        return if dir.is_nan() {
                            Some(FigureItem::Point(pos, color))
                        } else {
                            Some(FigureItem::Arrow(pos, dir, color))
                        };
                    }
                }
                IterMemory::Position(p0) => {
                    let p = self.iter.next();
                    if p.is_none() {
                        return None;
                    }
                    let p = p.unwrap().1;
                    let color = *self.color_map.entry(p.level).or_insert(Color::BLACK);
                    let pos = Point { x: p.x, y: p.y };
                    if p.tie {
                        self.memory = IterMemory::Vertex(pos, p.dir, color);
                        if self.aabb.contains(p0) || self.aabb.contains(pos) {
                            return Some(FigureItem::Tie(p0, pos, color));
                        }
                    } else {
                        self.memory = IterMemory::Position(pos);
                        if self.aabb.contains(pos) {
                            return if p.dir.is_nan() {
                                Some(FigureItem::Point(pos, color))
                            } else {
                                Some(FigureItem::Arrow(pos, p.dir, color))
                            };
                        }
                    }
                }
            }
        }
    }
}
