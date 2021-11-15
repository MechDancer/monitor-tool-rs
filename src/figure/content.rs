use super::{aabb::AABB, FigureItem};
use iced::{canvas::Geometry, Color, Point, Size};
use std::{
    collections::{vec_deque::Iter, HashMap, VecDeque},
    time::Instant,
};

mod cache;

use cache::TopicCache;

/// 话题内容，用于存储话题状态
pub struct TopicContent {
    pub sync_set: String, // 同步组
    pub layer: String,    // 图层

    capacity: usize,                    // 缓存容量
    queue: VecDeque<(Instant, Vertex)>, // 点数据
    color_map: HashMap<u8, Color>,      // 色彩映射

    cache: TopicCache,
}

/// 产生绘图对象的迭代器
pub struct Items<'a> {
    memory: IterMemory,
    iter: Iter<'a, (Instant, Vertex)>,
    color_map: &'a HashMap<u8, Color>,
}

/// 迭代绘图缓存
enum IterMemory {
    Vertex(Point, f32, Color),
    Position(Point),
}

/// 图形顶点
struct Vertex {
    pos: Point,
    dir: f32,
    level: u8,
    tie: bool,
}

impl TopicContent {
    pub fn draw(&mut self, bounds: Size) -> Geometry {
        let mut iter = self.queue.iter();
        if let Some(items) = iter.next().map(|(_, v)| Items {
            memory: IterMemory::Vertex(
                v.pos,
                v.dir,
                self.color_map.get(&v.level).map_or(Color::BLACK, |c| *c),
            ),
            iter,
            color_map: &self.color_map,
        }) {
            todo!()
        } else {
            todo!()
        }
    }

    /// 设置队列容量
    pub fn set_capacity(&mut self, len: usize) {
        if len != self.capacity {
            self.capacity = len;
            if self.queue.len() > len {
                self.truncate(len);
            }
        }
    }

    /// 设置关注长度
    pub fn set_focus(&mut self, len: usize) {
        self.cache.set_focus(len);
    }

    /// 获取时间范围
    pub fn begin(&self) -> Option<Instant> {
        self.queue.back().map(|(t, _)| *t)
    }

    /// 计算关注范围
    pub fn bound(&mut self) -> Option<AABB> {
        self.cache.bound(self.queue.iter().map(|(_, v)| v.pos))
    }

    /// 依最时间范围同步
    pub fn sync(&mut self, deadline: Instant) {
        let to_remove = self
            .queue
            .iter()
            .rev()
            .take_while(|(t, _)| t < &deadline)
            .count();
        if to_remove > 0 {
            self.truncate(self.queue.len() - to_remove);
        }
    }

    /// 移除部分数据并使缓存失效
    #[inline]
    fn truncate(&mut self, len: usize) {
        self.queue.truncate(len);
        self.cache.clear();
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            pos: Point::ORIGIN,
            dir: 0.0,
            level: 0,
            tie: false,
        }
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
                let color = self.color_map.get(&p.level).map_or(Color::BLACK, |c| *c);
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

#[test]
fn assert_size() {
    assert_eq!(16, std::mem::size_of::<Point>());
}
