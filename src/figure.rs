use iced::{canvas::Geometry, Point, Rectangle, Size, Vector};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    net::SocketAddr,
    time::{Duration, Instant},
};

mod aabb;
mod content;

use aabb::AABB;
use content::TopicContent;

/// 画面
#[derive(Default)]
pub struct Figure {
    auto_view: bool,
    view: View,

    topics: HashMap<TopicTitle, TopicContent>,
    visible_layers: HashSet<String>,
    sync_sets: HashMap<String, (HashSet<TopicTitle>, Duration)>,
}

/// 视野
#[derive(PartialEq, Clone, Copy, Debug)]
struct View {
    size: Size,
    center: Point,
    scale: f32,
}

/// 话题标题，用于区分话题
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct TopicTitle {
    title: String,
    source: SocketAddr,
}

impl Figure {
    /// 画图
    pub fn draw(&mut self, bounds: Size, available_bounds: Size) -> (Rectangle, Vec<Geometry>) {
        // 各组同步
        self.sync(Instant::now());
        self.view.size = bounds;
        // 计算自动范围
        if self.auto_view {
            if let Some(aabb) = self.aabb() {
                self.view.center = aabb.center();

                let Size { width, height } = aabb.size();
                let new = f32::min(
                    available_bounds.width / width,
                    available_bounds.height / height,
                );
                if new.is_finite() {
                    self.view.scale = new;
                }
            }
        }
        // 计算对角线
        let diagonal = Vector {
            x: bounds.width,
            y: bounds.height,
        } * (1.0 / self.view.scale);
        // 写入配置并绘制
        let geometries = self
            .topics
            .values_mut()
            .filter(|content| self.visible_layers.contains(&content.layer))
            .filter_map(|content| {
                content.set_config(self.view);
                content.draw()
            })
            .collect();
        (
            Rectangle {
                x: self.view.center.x - diagonal.x * 0.5,
                y: self.view.center.y + diagonal.y * 0.5,
                width: diagonal.x,
                height: diagonal.y,
            },
            geometries,
        )
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
            .filter(|content| self.visible_layers.contains(&content.layer))
            .filter_map(|content| content.aabb())
            .reduce(|sum, it| sum + it)
    }
}

impl Default for View {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl View {
    const DEFAULT: Self = Self {
        size: Size {
            width: 640.0,
            height: 480.0,
        },
        center: Point::ORIGIN,
        scale: 1.0,
    };
}

impl Display for TopicTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.title, self.source)
    }
}
