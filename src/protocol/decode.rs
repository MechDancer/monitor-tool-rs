use std::{net::SocketAddr, time::Duration};

use crate::{figure::Figure, protocol::Visible, Camera};

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
    // 解析图层
    let layers = read_by_tails!(buf => Layers);
}

struct SyncSets<'a>(&'a [u16], &'a [u8]);

struct Layers<'a>(&'a [u16], &'a [u8]);

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

impl<'a> SyncSets<'a> {
    fn get(&self, i: usize) -> (&'a Duration, &'a str) {
        const LEN: usize = std::mem::size_of::<Duration>();
        let begin = if i == 0 { 0 } else { self.0[i - 1] as usize };
        let slice = &self.1[begin..self.0[i] as usize];
        unsafe {
            (
                &*(slice.as_ptr() as *const Duration),
                std::str::from_utf8_unchecked(&slice[LEN..]),
            )
        }
    }
}

impl<'a> Layers<'a> {
    fn get(&self, i: usize) -> (&'a Visible, &'a str) {
        const LEN: usize = std::mem::size_of::<Visible>();
        let begin = if i == 0 { 0 } else { self.0[i - 1] as usize };
        let slice = &self.1[begin..self.0[i] as usize];
        unsafe {
            (
                &*(slice.as_ptr() as *const Visible),
                std::str::from_utf8_unchecked(&slice[LEN..]),
            )
        }
    }
}
