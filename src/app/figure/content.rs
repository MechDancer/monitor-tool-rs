use super::{aabb::AABB, View};
use crate::Vertex;
use iced::{canvas::Geometry, Color, Point};
use std::time::Instant;

mod buffer;
mod cache;
mod items;

pub(crate) use buffer::TopicBuffer;
use cache::TopicCache;
use items::Items;

#[derive(Default)]
pub(crate) struct TopicContent {
    pub layer: String,   // 图层
    buffer: TopicBuffer, // 话题的数据缓存
    cache: TopicCache,   // 话题的图形缓存
}

/// 单个绘图对象
#[derive(Clone, Copy)]
enum FigureItem {
    End(Point),
    Point(Point, Color),
    Arrow(Point, f32, Color),
    Circle(Point, f32, Color),
}

impl TopicContent {
    /// 构造快照
    pub fn snapshot(&self) -> TopicBuffer {
        self.buffer.clone()
    }

    /// 重绘
    #[inline]
    pub fn redraw(&mut self) {
        self.cache.redraw();
    }

    /// 设置队列容量
    #[inline]
    pub fn set_capacity(&mut self, len: usize) {
        if self.buffer.capacity != len {
            self.buffer.capacity = len;
            if self.buffer.queue.len() > len {
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
        if Some(color) != self.buffer.color_map.insert(i, color) {
            self.cache.redraw();
        }
    }

    /// 获取时间范围
    #[inline]
    pub fn begin(&self) -> Option<Instant> {
        self.buffer.queue.back().map(|(t, _)| *t)
    }

    /// 计算关注范围
    #[inline]
    pub fn aabb(&mut self) -> Option<AABB> {
        self.cache.aabb(self.buffer.queue.iter().map(|(_, v)| *v))
    }

    /// 画图
    #[inline]
    pub fn draw(&mut self, view: View, aabb: AABB) -> Option<Geometry> {
        Items::new(
            &self.buffer.queue,
            &mut self.buffer.color_map,
            view.center,
            aabb,
        )
        .map(|items| self.cache.draw(items, view.size, view.scale))
    }

    /// 向队列添加一组点
    pub fn extend_from_slice(&mut self, time: Instant, v: &[Vertex]) {
        for v in v {
            if let Some((t, v0)) = self.buffer.queue.front_mut() {
                if v0 == v {
                    *t = time;
                    continue;
                }
            }
            if self.buffer.queue.len() >= self.buffer.capacity {
                self.buffer.queue.pop_back();
            }
            self.buffer.queue.push_front((time, *v));
        }
        self.cache.clear();
    }

    /// 从队列移除所有点
    pub fn clear(&mut self) {
        self.buffer.queue.clear();
        self.cache.clear();
    }

    /// 依时间范围同步
    pub fn sync(&mut self, deadline: Instant) {
        let to_remove = self
            .buffer
            .queue
            .iter()
            .rev()
            .take_while(|(t, _)| t < &deadline)
            .count();
        if to_remove > 0 {
            self.truncate(self.buffer.queue.len() - to_remove);
        }
    }

    /// 移除部分数据并使缓存失效
    #[inline]
    fn truncate(&mut self, len: usize) {
        self.buffer.queue.truncate(len);
        self.cache.clear();
    }
}
