use iced::Color;
use nalgebra::Vector2;
use std::{
    collections::{vec_deque::Iter, HashMap, VecDeque},
    time::Instant,
};

/// 话题内容，用于存储话题状态
pub struct TopicContent {
    sync_set: String,

    timestamp: Instant,                // 最后更新时间
    queue: VecDeque<(Instant, Point)>, // 点数据
    color_map: HashMap<u8, Color>,     // 色彩映射
}

pub struct Points<'a> {
    memory: IterMemory,
    iter: Iter<'a, (Instant, Point)>,
    color_map: &'a HashMap<u8, Color>,
}

pub enum FigureItem {
    Point(Vector2<f32>, Color),
    Arrow(Vector2<f32>, f32, Color),
    Tie(Vector2<f32>, Vector2<f32>, Color),
}

enum IterMemory {
    Point(Vector2<f32>, f32, Color),
    Position(Vector2<f32>),
}

/// 图形顶点
struct Point {
    pos: Vector2<f32>,
    dir: f32,
    level: u8,
    tie: bool,
}

impl TopicContent {
    pub fn update_from_this(&self, time: Instant) -> bool {
        time < self.timestamp
    }

    pub fn upgrade_from_this<'a>(&'a self) -> Option<Points<'a>> {
        let mut iter = self.queue.iter();
        iter.next().map(|(_, p)| Points {
            memory: IterMemory::Point(
                p.pos,
                p.dir,
                self.color_map.get(&p.level).map_or(Color::BLACK, |c| *c),
            ),
            iter,
            color_map: &self.color_map,
        })
    }

    /// 移除到最大存量，返回剩余点的最早时间
    pub fn sync_by_size(&mut self, max_len: usize) -> Option<Instant> {
        if self.queue.len() > max_len {
            self.queue.truncate(max_len);
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
        self.queue.truncate(self.queue.len() - to_remove);
    }
}

impl Default for TopicContent {
    fn default() -> Self {
        Self {
            sync_set: Default::default(),
            timestamp: Instant::now(),
            queue: Default::default(),
            color_map: Default::default(),
        }
    }
}

impl Default for Point {
    fn default() -> Self {
        Self {
            pos: Vector2::new(0.0, 0.0),
            dir: 0.0,
            level: 0,
            tie: false,
        }
    }
}

impl<'a> Iterator for Points<'a> {
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
