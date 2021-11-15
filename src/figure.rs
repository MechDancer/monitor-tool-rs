use iced::{Color, Point, Rectangle};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    net::IpAddr,
    time::{Duration, Instant},
};

mod aabb;
mod content;

pub use content::TopicContent;

/// 画面
pub struct Figure {
    topics: HashMap<TopicTitle, TopicContent>,
    layers: HashMap<String, (HashSet<TopicTitle>, bool)>,
    sync_sets: HashMap<String, (HashSet<TopicTitle>, Duration)>,
}

pub enum FigureItem {
    Point(Point, Color),
    Arrow(Point, f32, Color),
    Tie(Point, Point, Color),
}

/// 话题标题，用于区分话题
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct TopicTitle {
    title: String,
    source: IpAddr,
}

impl Display for TopicTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.title, self.source)
    }
}

impl Figure {
    /// 同步
    fn sync(&mut self, now: Instant) {
        for (set, life_time) in self.sync_sets.values_mut() {
            // 按当前时间计算期限
            let deadline0 = now.checked_sub(*life_time);
            // 按数量消除并计算期限
            let deadline1 = set.iter().filter_map(|t| self.topics[t].begin()).min();
            // 合并期限
            if let Some(deadline) = match (deadline0, deadline1) {
                (None, None) => None,
                (Some(t), None) => Some(t),
                (None, Some(t)) => Some(t),
                (Some(t0), Some(t1)) => Some(std::cmp::min(t0, t1)),
            } {
                // 按期限消除
                for t in set.iter() {
                    self.topics.get_mut(t).unwrap().sync(deadline)
                }
            }
        }
    }

    /// 计算范围
    fn bound(&mut self) -> Option<Rectangle> {
        self.topics
            .values_mut()
            .filter(|content| self.layers[&content.layer].1)
            .filter_map(|content| content.bound())
            .reduce(|sum, it| sum + it)
            .map(|aabb| aabb.to_rectangle())
    }
}
