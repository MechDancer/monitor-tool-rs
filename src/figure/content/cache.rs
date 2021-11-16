use super::{FigureItem, Items, View, AABB};
use iced::{
    canvas::{Cache, Geometry, Path, Stroke},
    Point, Vector,
};

#[derive(Default)]
pub(super) struct TopicCache {
    focus_len: usize,
    bound: Bound,

    config: View,
    cache: Cache,
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
    pub fn aabb(&mut self, iter: impl Iterator<Item = Point>) -> Option<AABB> {
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
        let View {
            size,
            center,
            scale,
        } = self.config;
        let items = items.collect::<Vec<_>>();
        self.cache.draw(size, |frame| {
            frame.translate(frame.center() - Point::ORIGIN);
            for item in items.iter().copied() {
                match item {
                    FigureItem::Point(p, color) => {
                        let Vector { x, y } = (p - center) * scale;
                        frame.fill(&Path::circle(Point { x, y: -y }, 1.0), color);
                    }
                    FigureItem::Arrow(p, d, color) => {
                        let Vector { x, y } = (p - center) * scale;
                        let p = Point { x, y: -y };
                        let (sin, cos) = d.sin_cos();
                        let d = Vector {
                            x: cos * 15.0,
                            y: sin * -15.0,
                        };
                        frame.fill(&Path::circle(p, 2.0), color);
                        frame.stroke(
                            &Path::line(p, p + d),
                            Stroke {
                                color,
                                width: 1.5,
                                ..Default::default()
                            },
                        );
                    }
                    FigureItem::Tie(p0, p1, color) => {
                        let Vector { x, y } = (p0 - center) * scale;
                        let p0 = Point { x, y: -y };

                        let Vector { x, y } = (p1 - center) * scale;
                        let p1 = Point { x, y: -y };

                        frame.stroke(
                            &Path::line(p0, p1),
                            Stroke {
                                color,
                                width: 1.5,
                                ..Default::default()
                            },
                        );
                    }
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
    pub fn set_config(&mut self, config: View) {
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
