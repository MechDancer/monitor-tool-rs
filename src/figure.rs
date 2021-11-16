use iced::{canvas::Geometry, Point, Size};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    net::IpAddr,
    time::{Duration, Instant},
};

mod aabb;
mod content;

use content::TopicContent;

use self::aabb::AABB;

/// 画面
#[derive(Default)]
pub struct Figure {
    center: Point, // 画面中心坐标
    scale: Scale,  // 缩放尺度

    topics: HashMap<TopicTitle, TopicContent>,
    layers: HashMap<String, (HashSet<TopicTitle>, bool)>,
    sync_sets: HashMap<String, (HashSet<TopicTitle>, Duration)>,
}

#[derive(PartialEq, Clone, Copy)]
struct Config {
    size: Size,
    center: Point,
    scale: f32,
}

/// 缩放尺度
enum Scale {
    Static(f32),
    Automatic,
}

/// 话题标题，用于区分话题
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct TopicTitle {
    title: String,
    source: IpAddr,
}

impl Figure {
    pub fn draw(&mut self, now: Instant, bounds: Size) -> Vec<Geometry> {
        self.sync(now);

        let config = Config {
            size: bounds,
            center: self.center,
            scale: match self.scale {
                Scale::Static(k) => k,
                Scale::Automatic => self
                    .aabb()
                    .and_then(|aabb| {
                        let Size { width, height } = aabb.size();
                        Some(f32::min(bounds.width / width, bounds.height / height))
                            .filter(|k| k.is_finite())
                    })
                    .unwrap_or(1.0),
            },
        };
        self.topics
            .values_mut()
            .filter_map(|content| {
                content.set_config(config);
                content.draw()
            })
            .collect()
    }

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
    fn aabb(&mut self) -> Option<AABB> {
        self.topics
            .values_mut()
            .filter(|content| self.layers[&content.layer].1)
            .filter_map(|content| content.bound())
            .reduce(|sum, it| sum + it)
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self::Static(1.0)
    }
}

impl Display for TopicTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.title, self.source)
    }
}
