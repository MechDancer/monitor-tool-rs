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
pub(crate) use content::TopicContent;

/// 画面
#[derive(Default)]
pub(crate) struct Figure {
    pub auto_view: bool,
    pub view: View,

    topics: HashMap<TopicTitle, TopicContent>,
    visible_layers: HashSet<String>,
    sync_sets: HashMap<String, (HashSet<TopicTitle>, Duration)>,
}

/// 视野
#[derive(PartialEq, Clone, Copy, Debug)]
pub(crate) struct View {
    pub size: Size,
    pub center: Point,
    pub scale: f32,
}

/// 话题标题，用于区分话题
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct TopicTitle {
    pub title: String,
    pub source: SocketAddr,
}

impl Figure {
    /// 放缩
    pub fn zoom(&mut self, level: f32, pos: Point, bounds: Size) {
        // 关闭自动
        self.auto_view = false;
        // 计算尺度
        let k = if level > 0.0 {
            1.1f32.powf(level)
        } else {
            0.9f32.powf(-level)
        };
        self.view.scale *= k;
        // 计算中心偏移
        let c = Point {
            x: bounds.width * 0.5,
            y: bounds.height * 0.5,
        };
        let Vector { x, y } = (pos - c) * ((k - 1.0) / self.view.scale);
        self.view.center = self.view.center + Vector { x, y: -y };
        // 打日志
        println!("scale = {}", self.view.scale);
    }

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
            .filter(|content| check_visible(&self.visible_layers, &content.layer))
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

    /// 设置同步组时限
    pub fn set_life_time(&mut self, sync_set: impl ToString, life_time: Duration) {
        self.sync_sets.entry(sync_set.to_string()).or_default().1 = life_time;
    }

    /// 设置图层可见性
    pub fn set_visible(&mut self, layer: impl ToString, visible: bool) {
        if visible {
            self.visible_layers.insert(layer.to_string());
        } else {
            self.visible_layers.remove(&layer.to_string());
        }
    }

    /// 重新关联同步组
    pub fn update_sync_set(&mut self, sync_set: &String, title: TopicTitle) {
        let set = &mut self.sync_sets.get_mut(sync_set).unwrap().0;
        if sync_set.is_empty() {
            set.remove(&title);
        } else {
            set.insert(title);
        }
    }

    /// 获取话题对象
    pub fn topic_mut<'a>(&'a mut self, title: TopicTitle) -> &'a mut TopicContent {
        self.topics.entry(title).or_default()
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
            .filter(|content| check_visible(&self.visible_layers, &content.layer))
            .filter_map(|content| content.aabb())
            .reduce(|sum, it| sum + it)
    }
}

#[inline]
fn check_visible(set: &HashSet<String>, layer: &String) -> bool {
    layer.is_empty() || set.contains(layer)
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
