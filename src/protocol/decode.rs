use super::Visibility;
use crate::{Figure, Vertex};
use palette::rgb::channels::Argb;
use palette::{Pixel, Srgba};
use std::time::{Duration, Instant};

mod sync_sets_and_layers;

use sync_sets_and_layers::*;

macro_rules! read {
    ($buf:expr => $ty:ty) => {{
        const LEN: usize = std::mem::size_of::<$ty>();
        if $buf.len() >= LEN {
            let ptr = $buf.as_ptr() as *const $ty;
            $buf = &$buf[LEN..];
            Some(unsafe { &*ptr })
        } else {
            None
        }
    }};
    ($buf:expr => $ty:ty; $n:expr) => {{
        const LEN: usize = std::mem::size_of::<$ty>();
        let n = $n as usize;
        let len = n * LEN;
        if $buf.len() >= len {
            let slice = unsafe { std::slice::from_raw_parts($buf.as_ptr() as *const $ty, n) };
            $buf = &$buf[len..];
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

pub(crate) fn decode(figure: &mut Figure, time: Instant, mut buf: &[u8]) {
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
            Visibility::NothingToDo => {}
            Visibility::Visible => figure.set_visible(layer, true),
            Visibility::Invisible => figure.set_visible(layer, false),
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
            match read!(buf => u8; len) {
                Some(slice) => unsafe { std::str::from_utf8_unchecked(slice) },
                None => return,
            }
        };
        // 更新同步组
        match read!(buf => u16) {
            Some(0) => {}
            Some(i) => {
                let (sync_set, _) = sync_sets.get(*i as usize - 1);
                figure.update_sync_set(sync_set, title.to_string());
            }
            None => return,
        }
        // 更新话题内容
        let topic = figure.put_topic(title.to_string());
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
                        let rgba = unsafe { *(color[1..].as_ptr() as *const u32) };
                        let rgba: [f32; 4] = Srgba::from_u32::<Argb>(rgba).into_format().into_raw();
                        topic.set_color(level, rgba.into());
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
