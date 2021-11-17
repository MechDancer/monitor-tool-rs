use super::{aabb::AABB, View};
use crate::Vertex;
use iced::{canvas::Geometry, Color, Point};
use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
};

mod cache;
mod items;

use cache::TopicCache;
use items::Items;

/// 话题内容，用于存储话题状态
pub(super) struct TopicContent {
    pub sync_set: String, // 同步组
    pub layer: String,    // 图层

    capacity: usize,                    // 缓存容量
    queue: VecDeque<(Instant, Vertex)>, // 点数据
    color_map: HashMap<u8, Color>,      // 色彩映射

    cache: TopicCache, // 话题的完整缓存
}

/// 单个绘图对象
#[derive(Clone, Copy)]
enum FigureItem {
    Point(Point, Color),
    Arrow(Point, f32, Color),
    Tie(Point, Point, Color),
}

impl TopicContent {
    pub fn new(sync_set: impl ToString, layer: impl ToString) -> Self {
        Self {
            sync_set: sync_set.to_string(),
            layer: layer.to_string(),

            capacity: 10000,
            queue: Default::default(),
            color_map: Default::default(),

            cache: Default::default(),
        }
    }

    /// 设置队列容量
    #[inline]
    pub fn set_capacity(&mut self, len: usize) {
        if self.capacity != len {
            self.capacity = len;
            if self.queue.len() > len {
                self.truncate(len);
            }
        }
    }

    /// 设置关注长度
    #[inline]
    pub fn set_focus(&mut self, len: usize) {
        self.cache.set_focus(len);
    }

    /// 设置级别颜色
    #[inline]
    pub fn set_color(&mut self, i: u8, color: Color) {
        if Some(color) != self.color_map.insert(i, color) {
            self.cache.redraw();
        }
    }

    /// 设置变换
    #[inline]
    pub fn set_config(&mut self, config: View) {
        self.cache.set_config(config);
    }

    /// 获取时间范围
    #[inline]
    pub fn begin(&self) -> Option<Instant> {
        self.queue.back().map(|(t, _)| *t)
    }

    /// 计算关注范围
    #[inline]
    pub fn aabb(&mut self) -> Option<AABB> {
        self.cache
            .aabb(self.queue.iter().map(|(_, v)| Point { x: v.x, y: v.y }))
    }

    /// 画图
    #[inline]
    pub fn draw(&mut self) -> Option<Geometry> {
        Items::new(&self.queue, &mut self.color_map).map(|items| self.cache.draw(items))
    }

    /// 向队列添加一组点
    pub fn push(&mut self, time: Instant, v: impl IntoIterator<Item = Vertex>) {
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
