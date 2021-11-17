use crate::{
    figure::{Figure, TopicTitle},
    protocol::Visible,
    Camera, Vertex, RGBA,
};
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

mod sync_sets_and_layers;

use iced::Color;
use sync_sets_and_layers::*;

macro_rules! read {
    ($buf:expr => $ty:ty) => {
        if $buf.len() >= std::mem::size_of::<$ty>() {
            let ptr = $buf.as_ptr() as *const $ty;
            $buf = &$buf[std::mem::size_of::<$ty>()..];
            Some(unsafe { &*ptr })
        } else {
            None
        }
    };
    ($buf:expr => $ty:ty; $n:expr) => {{
        let n = $n as usize;
        if $buf.len() >= n * std::mem::size_of::<$ty>() {
            let slice = unsafe { std::slice::from_raw_parts($buf.as_ptr() as *const $ty, n) };
            $buf = &$buf[std::mem::size_of::<$ty>()..];
            Some(slice)
        } else {
            None
        }
    }};
}

macro_rules! read_by_tails {
    ($buf:expr => $ty:expr) => {
        match read!($buf => u16) {
            Some(0) => $ty(&[], &[]),
            Some(n) => match read!($buf => u16; *n) {
                Some(tails) => match read!($buf => u8; tails[*n as usize - 1]) {
                    Some(slice) => $ty(tails, slice),
                    None => return,
                },
                None => return,
            },
            None => return,
        }
    };
}

pub(crate) fn decode(figure: &mut Figure, time: Instant, src: SocketAddr, mut buf: &[u8]) {
    // 解析摄像机
    match read!(buf => Camera) {
        Some(camera) => update_camera(figure, camera),
        None => return,
    }
    // 解析同步组
    let sync_sets = read_by_tails!(buf => SyncSets);
    for i in 0..sync_sets.len() {
        let (sync_set, life_time) = sync_sets.get(i);
        if life_time != &Duration::ZERO {
            figure.set_life_time(sync_set, *life_time);
        }
    }
    // 解析图层
    let layers = read_by_tails!(buf => Layers);
    for i in 0..layers.len() {
        let (layer, visible) = layers.get(i);
        match *visible {
            Visible::NothingToDo => {}
            Visible::Visible => figure.set_visible(layer, true),
            Visible::Invisible => figure.set_visible(layer, false),
        }
    }
    // 解析话题
    loop {
        // 构造话题标题
        let title = {
            let len = match read!(buf => u16) {
                Some(len) => *len as usize,
                None => return,
            };
            let title = match read!(buf => u8; len) {
                Some(slice) => unsafe { std::str::from_utf8_unchecked(slice) },
                None => return,
            };
            TopicTitle {
                title: title.to_string(),
                source: src,
            }
        };
        // 更新同步组
        match read!(buf => u16) {
            Some(0) => {}
            Some(i) => {
                let (sync_set, _) = sync_sets.get(*i as usize - 1);
                figure.update_sync_set(&sync_set.to_string(), title.clone());
            }
            None => return,
        }
        // 更新话题内容
        let topic = figure.topic_mut(title);
        // 更新图层
        match read!(buf => u16) {
            Some(0) => {}
            Some(i) => topic.layer = layers.get(*i as usize - 1).0.to_string(),
            None => return,
        }
        // 清除缓存
        match read!(buf => bool) {
            Some(false) => {}
            Some(true) => topic.clear(),
            None => return,
        }
        // 更新容量
        match read!(buf => u32) {
            Some(0) => {}
            Some(n) => topic.set_capacity(*n as usize),
            None => return,
        }
        // 更新关注数量
        match read!(buf => u32) {
            Some(0) => {}
            Some(n) => topic.set_focus(*n as usize),
            None => return,
        }
        // 更新颜色
        match read!(buf => u16) {
            Some(0) => {}
            Some(n) => match read!(buf => [u8; 5]; *n) {
                Some(colors) => {
                    for color in colors {
                        let level = color[0];
                        let rgba = unsafe { *(color[1..].as_ptr() as *const RGBA) };
                        topic.set_color(level, rgba_to_color(rgba));
                    }
                }
                None => return,
            },
            None => return,
        }
        // 存入点
        match read!(buf => u16) {
            Some(0) => {}
            Some(n) => match read!(buf => Vertex; *n) {
                Some(vertexs) => topic.extend_from_slice(time, vertexs),
                None => return,
            },
            None => return,
        }
    }
}

fn update_camera(figure: &mut Figure, camera: &Camera) {
    let Camera {
        x,
        y,
        scale_x,
        scale_y,
    } = camera;
    if x.is_normal() {
        figure.view.center.x = *x;
    }
    if y.is_normal() {
        figure.view.center.y = *y;
    }
    if scale_x.is_normal() && scale_y.is_normal() {
        if *scale_x == 0.0 || *scale_y == 0.0 {
            figure.auto_view = true;
        } else {
            figure.view.scale = f32::min(*scale_x, *scale_y);
        }
    }
}

#[inline]
fn rgba_to_color(rgba: RGBA) -> Color {
    Color {
        r: rgba.0 as f32 / 255.0,
        g: rgba.1 as f32 / 255.0,
        b: rgba.2 as f32 / 255.0,
        a: rgba.3 as f32 / 255.0,
    }
}
