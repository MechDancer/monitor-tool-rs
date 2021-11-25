use super::{super::convert, FigureItem, Vertex, AABB};
use crate::Shape::*;
use iced::{Color, Point};
use std::{
    collections::{vec_deque::Iter, HashMap, VecDeque},
    time::Instant,
};

/// 产生绘图对象的迭代器
pub(super) struct Items<'a> {
    memory: Option<TieMemory>,
    center: Point,
    aabb: AABB,
    iter: Iter<'a, (Instant, Vertex)>,
    color_map: &'a mut HashMap<u8, Color>,
}

struct TieMemory {
    pos: Point,
    inside: bool,
    color: Color,
}

impl<'a> Items<'a> {
    pub fn new(
        queue: &'a VecDeque<(Instant, Vertex)>,
        color_map: &'a mut HashMap<u8, Color>,
        center: Point,
        aabb: AABB,
    ) -> Option<Self> {
        if queue.is_empty() {
            None
        } else {
            Some(Items {
                memory: None,
                center,
                aabb,
                iter: queue.iter(),
                color_map,
            })
        }
    }

    #[inline]
    fn find_color(&mut self, level: u8) -> Color {
        *self.color_map.entry(level).or_insert(Color::BLACK)
    }
}

impl<'a> Iterator for Items<'a> {
    type Item = (Option<(Point, Color)>, FigureItem);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((_, v)) = self.iter.next() {
            let inside = self.aabb.contains(&v);
            let pos = convert(v.pos(), self.center);
            let tie = self.memory.take();
            if v.alpha > 0 {
                let mut color = self.find_color(v.level);
                color.a *= v.alpha as f32 / 255.0;
                self.memory = Some(TieMemory { pos, inside, color });
            }
            if inside {
                let color = self.find_color(v.level);
                let tie = tie.map(|mem| (mem.pos, mem.color));
                match v.shape {
                    Arrow => {
                        if v.extra.is_finite() {
                            return Some((tie, FigureItem::Arrow(pos, -v.extra, color)));
                        } else {
                            return Some((tie, FigureItem::Point(pos, color)));
                        }
                    }
                    Circle => {
                        if v.extra.is_normal() {
                            return Some((tie, FigureItem::Circle(pos, v.extra, color)));
                        }
                    }
                }
            } else if let Some(tie) = tie.filter(|mem| mem.inside).map(|mem| (mem.pos, mem.color)) {
                return Some((Some(tie), FigureItem::End(pos)));
            }
        }
        return None;
    }
}
