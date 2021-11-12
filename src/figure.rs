use iced::Color;
use nalgebra::Vector2;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    net::IpAddr,
    time::{Duration, Instant},
};

mod content;

pub use content::TopicContent;

/// 画面
pub struct Figure {
    topics: HashMap<TopicTitle, TopicContent>,
    layers: HashMap<String, HashSet<TopicTitle>>,
    sync_sets: HashMap<String, (HashSet<TopicTitle>, SyncInfo)>,
}

pub enum FigureItem {
    Point(Vector2<f32>, Color),
    Arrow(Vector2<f32>, f32, Color),
    Tie(Vector2<f32>, Vector2<f32>, Color),
}

/// 同步信息
pub struct SyncInfo {
    max_size: usize,
    life_time: Duration,
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
    /// 限时限量同步
    fn sync(&mut self, set: String, now: Instant) {
        let (set, info) = &self.sync_sets[&set];
        // 按当前时间计算期限
        let deadline0 = now.checked_sub(info.life_time);
        // 按数量消除并计算期限
        let deadline1 = set
            .iter()
            .filter_map(|t| self.topics.get_mut(t).unwrap().sync_by_size(info.max_size))
            .min();
        // 合并期限
        if let Some(deadline) = match (deadline0, deadline1) {
            (None, None) => None,
            (Some(t), None) => Some(t),
            (None, Some(t)) => Some(t),
            (Some(t0), Some(t1)) => Some(std::cmp::min(t0, t1)),
        } {
            // 按期限消除
            for t in set {
                self.topics.get_mut(t).unwrap().sync_by_time(deadline)
            }
        }
    }
}
