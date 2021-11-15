use super::AABB;
use iced::{Point, Vector};
use iced_graphics::Primitive;

#[derive(Default)]
pub(super) struct TopicCache {
    focus_len: usize,

    bound: Bound,
    primitive: Option<(Vector, f32, Primitive)>,
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

    #[inline]
    pub fn set_focus(&mut self, len: usize) {
        if self.focus_len != len {
            self.focus_len = len;
            self.rebound();
        }
    }

    #[inline]
    pub fn redraw(&mut self) {
        self.primitive = None;
    }

    #[inline]
    pub fn clear(&mut self) {
        self.rebound();
        self.redraw();
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
