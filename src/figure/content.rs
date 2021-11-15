use super::aabb::AABB;
use iced::{canvas::Geometry, Color, Point};
use std::{
    collections::{hash_map::Entry, vec_deque::Iter, HashMap, VecDeque},
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

    cache: TopicCache, // 话题的完整缓存
}

/// 图形顶点
pub struct Vertex {
    pos: Point,
    dir: f32,
    level: u8,
    tie: bool,
}

/// 产生绘图对象的迭代器
struct Items<'a> {
    memory: IterMemory,
    iter: Iter<'a, (Instant, Vertex)>,
    color_map: &'a mut HashMap<u8, Color>,
}

/// 单个绘图对象
#[derive(Clone, Copy)]
enum FigureItem {
    Point(Point, Color),
    Arrow(Point, f32, Color),
    Tie(Point, Point, Color),
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

impl TopicContent {
    /// 画图
    pub fn draw(&mut self) -> Option<Geometry> {
        let mut iter = self.queue.iter();
        iter.next().map(|(_, v)| {
            self.cache.draw(Items {
                memory: IterMemory::Vertex(
                    v.pos,
                    v.dir,
                    get_or_set!(self.color_map, v.level, Color::BLACK),
                ),
                iter,
                color_map: &mut self.color_map,
            })
        })
    }

    /// 设置队列容量
    pub fn set_capacity(&mut self, len: usize) {
        if self.capacity != len {
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

    /// 设置级别颜色
    pub fn set_color(&mut self, i: u8, color: Color) {
        if Some(color) != self.color_map.insert(i, color) {
            self.cache.redraw();
        }
    }

    /// 获取时间范围
    pub fn begin(&self) -> Option<Instant> {
        self.queue.back().map(|(t, _)| *t)
    }

    /// 计算关注范围
    pub fn bound(&mut self) -> Option<AABB> {
        self.cache.bound(self.queue.iter().map(|(_, v)| v.pos))
    }

    /// 向队列添加一点
    pub fn push(&mut self, time: Instant, v: impl Iterator<Item = Vertex>) {
        for v in v {
            if self.queue.len() >= self.capacity {
                self.queue.pop_back();
            }
            self.queue.push_front((time, v));
        }
        self.cache.clear();
    }

    /// 依时间范围同步
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

#[test]
fn assert_size() {
    assert_eq!(16, std::mem::size_of::<Point>());
}
