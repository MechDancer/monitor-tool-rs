use crate::figure::to_canvas;

use super::{FigureItem, Vertex, AABB};
use iced::{Color, Point};
use std::{
    collections::{vec_deque::Iter, HashMap, VecDeque},
    time::Instant,
};

/// 产生绘图对象的迭代器
pub(super) struct Items<'a> {
    memory: IterMemory,
    center: Point,
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
        center: Point,
        aabb: AABB,
    ) -> Option<Self> {
        let mut iter = queue.iter();
        iter.next().map(|(_, v)| Items {
            memory: IterMemory::Vertex(
                to_canvas(v.pos(), center),
                -v.dir,
                *color_map.entry(v.level).or_insert(Color::BLACK),
            ),
            center,
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
                    let v = p.unwrap().1;
                    let color = *self.color_map.entry(v.level).or_insert(Color::BLACK);
                    let pos = to_canvas(v.pos(), self.center);
                    if v.tie {
                        self.memory = IterMemory::Vertex(pos, -v.dir, color);
                        if self.aabb.contains(p0) || self.aabb.contains(pos) {
                            return Some(FigureItem::Tie(p0, pos, color));
                        }
                    } else {
                        self.memory = IterMemory::Position(pos);
                        if self.aabb.contains(pos) {
                            return if v.dir.is_nan() {
                                Some(FigureItem::Point(pos, color))
                            } else {
                                Some(FigureItem::Arrow(pos, v.dir, color))
                            };
                        }
                    }
                }
            }
        }
    }
}
