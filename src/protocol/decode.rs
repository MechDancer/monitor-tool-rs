use crate::{figure::Figure, protocol::Visible, Camera};
use std::{net::SocketAddr, time::Duration};

mod sync_sets_and_layers;

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

pub(crate) fn decode(figure: &mut Figure, src: SocketAddr, mut buf: &[u8]) {
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
    for i in 0..sync_sets.len() {
        let (layer, visible) = layers.get(i);
        match *visible {
            Visible::NothingToDo => {}
            Visible::Visible => figure.set_visible(layer, true),
            Visible::Invisible => figure.set_visible(layer, false),
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
