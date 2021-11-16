use iced::{canvas::Geometry, Point, Size};
use std::{
    collections::{HashMap, HashSet},
    f32::consts::FRAC_PI_2,
    fmt::Display,
    net::{SocketAddr, SocketAddrV4},
    str::FromStr,
    time::{Duration, Instant},
};

mod aabb;
mod content;

use content::TopicContent;

use self::{aabb::AABB, content::Vertex};

/// 画面
#[derive(Default)]
pub struct Figure {
    camera: Camera, // 视角

    topics: HashMap<TopicTitle, TopicContent>,
    visible_layers: HashSet<String>,
    sync_sets: HashMap<String, (HashSet<TopicTitle>, Duration)>,
}

#[derive(PartialEq, Clone, Copy)]
struct Config {
    size: Size,
    center: Point,
    scale: f32,
}

/// 变换
#[derive(Debug)]
enum Camera {
    Static(Point, f32),
    Automatic(Point, f32),
}

/// 话题标题，用于区分话题
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct TopicTitle {
    title: String,
    source: SocketAddr,
}

impl Figure {
    pub fn new() -> Self {
        let mut default = Self::default();
        let title = TopicTitle {
            title: "test".into(),
            source: SocketAddr::V4(SocketAddrV4::from_str("127.0.0.1:40000").unwrap()),
        };
        default.camera = Camera::Automatic(Point::ORIGIN, 1.0);
        default.visible_layers.insert("test".into());
        default
            .topics
            .insert(title.clone(), TopicContent::new("test", "test"));
        let topic = default.topics.get_mut(&title).unwrap();
        topic.push(
            Instant::now(),
            [
                Vertex {
                    pos: Point { x: 0.0, y: 0.0 },
                    dir: 0.0,
                    level: 0,
                    tie: true,
                },
                Vertex {
                    pos: Point { x: 10.0, y: 10.0 },
                    dir: FRAC_PI_2,
                    level: 0,
                    tie: true,
                },
            ],
        );
        topic.set_focus(100);
        default
    }

    /// 画图
    pub fn draw(&mut self, bounds: Size, available_bounds: Size) -> Vec<Geometry> {
        // 各组同步
        self.sync(Instant::now());
        // 创建配置
        let config = match self.camera {
            Camera::Static(center, scale) => Config {
                size: bounds,
                center,
                scale,
            },
            Camera::Automatic(mut center, mut scale) => {
                if let Some(aabb) = self.aabb() {
                    center = aabb.center();
                    let Size { width, height } = aabb.size();
                    let new = f32::min(
                        available_bounds.width / width,
                        available_bounds.height / height,
                    );
                    if new.is_finite() {
                        scale = new;
                    }
                }
                self.camera = Camera::Automatic(center, scale);
                Config {
                    size: bounds,
                    center,
                    scale,
                }
            }
        };
        // 写入配置并绘制
        self.topics
            .values_mut()
            .filter(|content| self.visible_layers.contains(&content.layer))
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
            .filter(|content| self.visible_layers.contains(&content.layer))
            .filter_map(|content| content.aabb())
            .reduce(|sum, it| sum + it)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::Static(Point::ORIGIN, 1.0)
    }
}

impl Display for TopicTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.title, self.source)
    }
}
