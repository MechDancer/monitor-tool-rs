use super::FigureItem;
use iced::{canvas::Geometry, Color, Point, Rectangle, Size};
use std::{
    collections::{vec_deque::Iter, HashMap, VecDeque},
    time::Instant,
};

/// 话题内容，用于存储话题状态
pub struct TopicContent {
    sync_set: String,

    queue: VecDeque<(Instant, Vertex)>, // 点数据
    color_map: HashMap<u8, Color>,      // 色彩映射
}

pub enum FocusMode {
    All(Rectangle),
    Last(Point),
    Background,
}

/// 产生绘图对象的迭代器
pub struct Items<'a> {
    memory: IterMemory,
    iter: Iter<'a, (Instant, Vertex)>,
    color_map: &'a HashMap<u8, Color>,
}

/// 迭代绘图缓存
enum IterMemory {
    Point(Point, f32, Color),
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
        if let Some(items) = iter.next().map(|(_, p)| Items {
            memory: IterMemory::Point(
                p.pos,
                p.dir,
                self.color_map.get(&p.level).map_or(Color::BLACK, |c| *c),
            ),
            iter,
            color_map: &self.color_map,
        }) {
            todo!()
        } else {
            todo!()
        }
    }

    /// 移除到最大存量，返回剩余点的最早时间
    pub fn sync_by_size(&mut self, max_len: usize) -> Option<Instant> {
        if self.queue.len() > max_len {
            self.truncate(max_len);
        }
        self.queue.back().map(|(t, _)| *t)
    }

    /// 移除到最早时间
    pub fn sync_by_time(&mut self, deadline: Instant) {
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
        // clear
    }
}

// impl Default for TopicContent {
//     fn default() -> Self {
//         Self {
//             sync_set: Default::default(),
//             queue: Default::default(),
//             color_map: Default::default(),
//             cache: Default::default(),
//         }
//     }
// }

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
            IterMemory::Point(pos, dir, color) => {
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
                    self.memory = IterMemory::Point(p.pos, p.dir, color);
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
