﻿use async_std::task;
use iced::{
    canvas::{Cache, Geometry},
    Color, Point, Rectangle, Size, Vector,
};
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

mod aabb;
mod border;
mod content;

use aabb::AABB;
use border::border;
pub(super) use border::{is_available, mark_cross};
pub(crate) use content::TopicContent;

use self::border::available_size;

/// 画面
pub(crate) struct Figure {
    update_time: Instant,
    print_time: bool,

    auto_view: bool,
    view: View,

    topics: HashMap<String, Option<Box<TopicContent>>>,
    visible_layers: HashSet<String>,
    sync_sets: HashMap<String, (HashSet<String>, Duration)>,

    border_cache: Cache,
}

/// 视野
#[derive(PartialEq, Clone, Copy, Debug)]
pub(crate) struct View {
    pub size: Size,
    pub center: Point,
    pub scale: f32,
}

macro_rules! unwrap {
    (     $wrapped:expr) => {
        $wrapped.as_ref().unwrap()
    };
    (mut; $wrapped:expr) => {
        $wrapped.as_mut().unwrap()
    };
}

impl Figure {
    pub fn new() -> Self {
        Self {
            update_time: Instant::now(),
            print_time: true,

            auto_view: false,
            view: View::DEFAULT,

            topics: Default::default(),
            visible_layers: Default::default(),
            sync_sets: Default::default(),

            border_cache: Default::default(),
        }
    }

    /// 设置视角
    pub fn set_view(&mut self, x: f32, y: f32, scale_x: f32, scale_y: f32) {
        let old = self.view;
        if x.is_finite() {
            self.view.center.x = x;
        }
        if y.is_finite() {
            self.view.center.y = y;
        }
        if scale_x.is_finite() && scale_y.is_finite() {
            if scale_x == 0.0 || scale_y == 0.0 {
                self.auto_view = true;
            } else {
                self.auto_view = false;
                self.view.scale = f32::min(scale_x, scale_y);
            }
        }
        if old != self.view {
            self.redraw();
        }
    }

    /// 放缩
    pub fn zoom(&mut self, level: f32, pos: Point, bounds: Rectangle) {
        self.auto_view = false;
        // 计算尺度
        if level.is_normal() {
            let k = (1.0 + level.signum() * 0.1).powf(level.abs());
            self.view.scale *= k;
            // 计算中心偏移
            let Vector { x, y } = (pos - bounds.center()) * ((k - 1.0) / self.view.scale);
            self.view.center = self.view.center + Vector { x, y: -y };
        }
        self.view.size = bounds.size();
        self.redraw();
    }

    /// 画图
    pub fn draw(&mut self) -> (Rectangle, Vec<Geometry>) {
        let time = Instant::now();
        // 各组同步
        self.sync(time);
        // 计算自动范围
        if self.auto_view {
            if let Some(aabb) = self.aabb() {
                let old = self.view;
                self.view.center = aabb.center();

                let Size { width, height } = aabb.size();
                let available_bounds = available_size(self.view.size);
                let new = f32::min(
                    available_bounds.width / width,
                    available_bounds.height / height,
                );
                if new.is_finite() {
                    self.view.scale = new;
                }
                if self.view != old {
                    self.redraw();
                }
            }
        }
        // 计算对角线
        let diagonal = Vector {
            x: self.view.size.width,
            y: self.view.size.height,
        } * (0.5 / self.view.scale);
        let aabb = AABB::foreach([
            to_canvas(self.view.center - diagonal, self.view.center),
            to_canvas(self.view.center + diagonal, self.view.center),
        ])
        .unwrap();
        let view = self.view;
        // 写入配置并绘制
        let tasks = self
            .topics
            .iter_mut()
            .filter(|(_, content)| check_visible(&self.visible_layers, content))
            .map(|(topic, content)| {
                let topic = topic.clone();
                let mut content = content.take().unwrap();
                task::spawn_blocking(move || {
                    let geometry = content.draw(view, aabb);
                    (topic, content, geometry)
                })
            })
            .collect::<Vec<_>>();
        let mut geometries = Vec::with_capacity(tasks.len() + 1);
        // 绘制边框
        geometries.push(
            self.border_cache
                .draw(self.view.size, |frame| border(frame, Color::BLACK)),
        );
        // 收集异步绘图结果
        geometries.extend(
            tasks
                .into_iter()
                .map(|handle| task::block_on(handle))
                .filter_map(|(name, content, geometry)| {
                    *self.topics.get_mut(&name).unwrap() = Some(content);
                    geometry
                }),
        );
        self.timer(time);
        (
            Rectangle {
                x: self.view.center.x - diagonal.x,
                y: self.view.center.y + diagonal.y,
                width: diagonal.x * 2.0,
                height: diagonal.y * 2.0,
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
    pub fn update_sync_set(&mut self, sync_set: &String, topic: String) {
        let set = &mut self.sync_sets.get_mut(sync_set).unwrap().0;
        if sync_set.is_empty() {
            set.remove(&topic);
        } else {
            set.insert(topic);
        }
    }

    /// 获取话题对象
    pub fn put_topic<'a>(&'a mut self, topic: impl ToString) -> &'a mut TopicContent {
        unwrap!(mut; self.topics
            .entry(topic.to_string())
            .or_insert(Some(Default::default())))
    }

    /// 获取话题对象
    pub fn get_topic<'a>(&'a mut self, topic: &String) -> Option<&'a mut Box<TopicContent>> {
        self.topics.get_mut(topic).map(|c| unwrap!(mut; c))
    }

    pub fn set_print_time(&mut self, value: bool) {
        self.print_time = value;
    }

    /// 同步
    fn sync(&mut self, time: Instant) {
        for (set, life_time) in self.sync_sets.values_mut() {
            // 按当前时间计算期限
            let deadline0 = time.checked_sub(*life_time);
            // 按数量消除并计算期限
            let deadline1 = set
                .iter()
                .filter_map(|t| unwrap!(self.topics[t]).begin())
                .min();
            // 合并期限
            if let Some(deadline) = match (deadline0, deadline1) {
                (None, None) => None,
                (Some(t), None) => Some(t),
                (None, Some(t)) => Some(t),
                (Some(t0), Some(t1)) => Some(std::cmp::max(t0, t1)),
            } {
                // 按期限消除
                for t in set.iter() {
                    unwrap!(mut; self.topics.get_mut(t).unwrap()).sync(deadline)
                }
            }
        }
    }

    /// 计算范围
    fn aabb(&mut self) -> Option<AABB> {
        self.topics
            .values_mut()
            .filter(|content| check_visible(&self.visible_layers, content))
            .filter_map(|content| unwrap!(mut; content).aabb())
            .reduce(|sum, it| sum + it)
    }

    #[inline]
    fn redraw(&mut self) {
        for content in self.topics.values_mut() {
            unwrap!(mut; content).redraw();
        }
    }

    /// 计时
    #[inline]
    fn timer(&mut self, time: Instant) {
        let last = std::mem::replace(&mut self.update_time, time);
        if self.print_time {
            println!(
                "period = {:?}, delay = {:?}",
                time - last,
                Instant::now() - time,
            );
        }
    }
}

#[inline]
fn check_visible(set: &HashSet<String>, content: &Option<Box<TopicContent>>) -> bool {
    let layer = &unwrap!(content).layer;
    layer.is_empty() || set.contains(layer)
}

#[inline]
fn to_canvas(p: Point, center: Point) -> Point {
    Point {
        x: p.x - center.x,
        y: center.y - p.y,
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
