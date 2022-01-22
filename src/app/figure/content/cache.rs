use crate::Vertex;

use super::{FigureItem, Items, AABB};
use iced::{
    canvas::{Cache, Geometry, Path, Stroke},
    Color, Point, Size, Vector,
};

#[derive(Default)]
pub(super) struct TopicCache {
    focus_len: usize,
    bound: Bound,
    cache: Cache,
}

#[derive(Debug)]
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
    pub fn aabb(&mut self, iter: impl Iterator<Item = Vertex>) -> Option<AABB> {
        match self.bound {
            Bound::Cached(aabb) => Some(aabb),
            Bound::CachedVoid => None,
            Bound::Suspectable(old) => {
                if let Some(new) = AABB::foreach_vertex(iter.take(self.focus_len)) {
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
            Bound::Invalid => AABB::foreach_vertex(iter.take(self.focus_len)).map(|aabb| {
                self.bound = Bound::Cached(aabb);
                self.redraw();
                aabb
            }),
        }
    }

    /// 画图
    pub fn draw(&mut self, items: Items, size: Size, scale: f32) -> Geometry {
        const MASS: usize = 2000;
        const WIDTH: f32 = 1.5;
        const D: f32 = 3.5;

        let items = items.collect::<Vec<_>>();
        let mass = items.len() > MASS;
        let d = if mass { WIDTH } else { D } / scale;
        let len_arrow = 15.0 / scale;
        let offset = Vector { x: d, y: d } * -0.5;

        self.cache.draw(size, |frame| {
            frame.translate(frame.center() - Point::ORIGIN);
            frame.scale(scale);

            let size = Size {
                width: d,
                height: d,
            };
            let mut stroke = Stroke {
                width: WIDTH,
                ..Default::default()
            };
            let min = 0.1 / scale.powi(2);
            for (tie, item) in items.iter().copied() {
                match item {
                    FigureItem::End(p) => {
                        draw_tie(tie, mass, p, min, |p0, color| {
                            stroke.color = color;
                            frame.stroke(&Path::line(p0, p), stroke);
                        });
                    }
                    FigureItem::Point(p, color) => {
                        let tied = draw_tie(tie, mass, p, min, |p0, color| {
                            stroke.color = color;
                            frame.stroke(&Path::line(p0, p), stroke);
                        });
                        // 小规模时一定画点
                        // 大规模时如果连了线就不画点
                        if !mass || !tied {
                            frame.fill_rectangle(p + offset, size, color);
                        }
                    }
                    FigureItem::Arrow(p, d, color) => {
                        draw_tie(tie, mass, p, min, |p0, color| {
                            stroke.color = color;
                            frame.stroke(&Path::line(p0, p), stroke);
                        });
                        if !mass {
                            frame.fill_rectangle(p + offset, size, color)
                        }
                        let (sin, cos) = d.sin_cos();
                        let d = Vector { x: cos, y: sin } * len_arrow;
                        stroke.color = color;
                        frame.stroke(&Path::line(p, p + d), stroke);
                    }
                    FigureItem::Circle(c, r, color) => {
                        draw_tie(tie, mass, c, min, |p0, color| {
                            stroke.color = color;
                            frame.stroke(&Path::line(p0, c), stroke);
                        });
                        stroke.color = color;
                        frame.stroke(&Path::circle(c, r), stroke);
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

fn draw_tie(
    tie: Option<(Point, Color)>,
    mass: bool,
    p: Point,
    min: f32,
    f: impl FnOnce(Point, Color),
) -> bool {
    if let Some((p0, color)) = tie {
        let tied = mass || {
            let v = p0 - p;
            (v.x.powi(2) + v.y.powi(2)) > min
        };
        if tied {
            f(p0, color);
        }
        tied
    } else {
        false
    }
}
