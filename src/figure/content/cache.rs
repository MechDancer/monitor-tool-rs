use super::{FigureItem, Items, AABB};
use iced::{
    canvas::{Cache, Geometry},
    Point, Size, Vector,
};

#[derive(Default)]
pub(super) struct TopicCache {
    focus_len: usize,
    bound: Bound,

    config: Config,
    cache: Cache,
}

#[derive(PartialEq)]
pub(super) struct Config {
    size: Size,
    translation: Vector,
    scale: f32,
}

enum Bound {
    Cached(AABB),
    CachedVoid,
    Suspectable(AABB),
    Invalid,
}

impl Default for Bound {
    fn default() -> Self {
        Self::Invalid
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Config {
    const DEFAULT: Self = Self {
        size: Size {
            width: 640.0,
            height: 480.0,
        },
        translation: Vector { x: 0.0, y: 0.0 },
        scale: 1.0,
    };
}

impl TopicCache {
    /// 计算范围
    pub fn bound(&mut self, iter: impl Iterator<Item = Point>) -> Option<AABB> {
        match self.bound {
            Bound::Cached(aabb) => Some(aabb),
            Bound::CachedVoid => None,
            Bound::Suspectable(old) => {
                if let Some(new) = AABB::foreach(iter.take(self.focus_len)) {
                    self.bound = Bound::Cached(new);
                    if new != old {
                        self.redraw();
                    }
                    Some(new)
                } else {
                    self.bound = Bound::CachedVoid;
                    self.redraw();
                    None
                }
            }
            Bound::Invalid => AABB::foreach(iter.take(self.focus_len)).map(|aabb| {
                self.bound = Bound::Cached(aabb);
                self.redraw();
                aabb
            }),
        }
    }

    /// 画图
    pub fn draw(&mut self, items: Items) -> Geometry {
        let Config {
            size,
            translation,
            scale,
        } = self.config;
        self.cache.draw(size, move |frame| {
            frame.translate(translation);
            frame.scale(scale);

            use FigureItem::*;
            for item in items {
                match item {
                    Point(p, color) => {}
                    Arrow(p, d, color) => {}
                    Tie(p0, p1, color) => {}
                }
            }
        })
    }

    #[inline]
    pub fn set_focus(&mut self, len: usize) {
        if self.focus_len != len {
            self.focus_len = len;
            self.rebound();
        }
    }

    #[inline]
    pub fn set_config(&mut self, config: Config) {
        if self.config != config {
            self.config = config;
            self.redraw();
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.rebound();
        self.redraw();
    }

    #[inline]
    pub fn redraw(&mut self) {
        self.cache.clear();
    }

    #[inline]
    fn rebound(&mut self) {
        match self.bound {
            Bound::Cached(r) => self.bound = Bound::Suspectable(r),
            Bound::CachedVoid => self.bound = Bound::Invalid,
            _ => {}
        };
    }
}
