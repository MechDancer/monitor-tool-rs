use crate::{Shape, Vertex};
use iced::{Point, Size};
use std::cmp::Ordering::*;

/// 用外边界表示的范围盒子
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct AABB {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

impl From<Point> for AABB {
    /// 从一个点生成无限小的盒
    fn from(p: Point) -> Self {
        Self {
            min_x: p.x,
            max_x: p.x,
            min_y: p.y,
            max_y: p.y,
        }
    }
}

impl From<Vertex> for AABB {
    /// 从一个点生成无限小的盒
    fn from(v: Vertex) -> Self {
        match v.shape {
            Shape::Arrow => Self {
                min_x: v.x,
                max_x: v.x,
                min_y: v.y,
                max_y: v.y,
            },
            Shape::Circle => Self {
                min_x: v.x - v.extra,
                max_x: v.x + v.extra,
                min_y: v.y - v.extra,
                max_y: v.y + v.extra,
            },
        }
    }
}

impl AABB {
    /// 计算一组点的 AABB 盒
    pub fn foreach(iter: impl IntoIterator<Item = Point>) -> Option<Self> {
        let mut iter = iter.into_iter();
        iter.next().map(|front| {
            let mut aabb = AABB::from(front);
            iter.for_each(|p| aabb.absorb(p));
            aabb
        })
    }

    /// 计算一组点的 AABB 盒
    pub fn foreach_vertex(iter: impl IntoIterator<Item = Vertex>) -> Option<Self> {
        let mut iter = iter.into_iter();
        iter.next().map(|front| {
            let mut aabb = AABB::from(front);
            iter.for_each(|v| aabb += Self::from(v));
            aabb
        })
    }

    /// 计算 [`Size`]
    #[inline]
    pub fn size(&self) -> Size {
        Size {
            width: self.max_x - self.min_x,
            height: self.max_y - self.min_y,
        }
    }

    /// 计算中心点
    #[inline]
    pub fn center(&self) -> Point {
        Point {
            x: (self.min_x + self.max_x) / 2.0,
            y: (self.min_y + self.max_y) / 2.0,
        }
    }

    /// 判断包含关系
    #[inline]
    pub fn contains(&self, v: &Vertex) -> bool {
        match v.shape {
            Shape::Arrow => {
                self.min_x <= v.x && v.x <= self.max_x && self.min_y <= v.y && v.y <= self.max_y
            }
            Shape::Circle => {
                let x = if v.x < self.min_x {
                    Less
                } else if v.x <= self.max_x {
                    Equal
                } else {
                    Greater
                };
                let y = if v.y < self.min_y {
                    Less
                } else if v.y <= self.max_y {
                    Equal
                } else {
                    Greater
                };
                #[inline]
                fn check_coner(x0: f32, y0: f32, x1: f32, y1: f32, r: f32) -> bool {
                    (x0 - x1).powi(2) + (y0 - y1).powi(2) <= r.powi(2)
                }
                match (x, y) {
                    (Less, Less) => check_coner(self.min_x, self.min_y, v.x, v.y, v.extra),
                    (Less, Equal) => self.min_x <= v.x + v.extra,
                    (Less, Greater) => check_coner(self.min_x, self.max_y, v.x, v.y, v.extra),
                    (Equal, Less) => self.min_y <= v.y + v.extra,
                    (Equal, Equal) => true,
                    (Equal, Greater) => self.max_y >= v.y - v.extra,
                    (Greater, Less) => check_coner(self.max_x, self.min_y, v.x, v.y, v.extra),
                    (Greater, Equal) => self.max_x >= v.x - v.extra,
                    (Greater, Greater) => check_coner(self.max_x, self.max_y, v.x, v.y, v.extra),
                }
            }
        }
    }

    /// 判断是否相交
    #[inline]
    pub fn intersect(&self, others: Self) -> bool {
        self.min_x <= others.max_x
            && others.min_x <= self.max_x
            && self.min_y <= others.max_y
            && others.min_y <= self.max_y
    }

    /// 吸收一个点，可能扩大盒范围
    fn absorb(&mut self, p: Point) {
        if p.x > self.max_x {
            self.max_x = p.x;
        } else if p.x < self.min_x {
            self.min_x = p.x;
        }
        if p.y > self.max_y {
            self.max_y = p.y;
        } else if p.y < self.min_y {
            self.min_y = p.y;
        }
    }
}

impl std::ops::Add for AABB {
    type Output = Self;

    /// 合并两个范围
    #[inline]
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl std::ops::AddAssign for AABB {
    /// 吸收另一个范围
    fn add_assign(&mut self, rhs: Self) {
        if rhs.max_x > self.max_x {
            self.max_x = rhs.max_x;
        }
        if rhs.min_x < self.min_x {
            self.min_x = rhs.min_x;
        }
        if rhs.max_y > self.max_y {
            self.max_y = rhs.max_y;
        }
        if rhs.min_y < self.min_y {
            self.min_y = rhs.min_y;
        }
    }
}

#[test]
fn test_size() {
    const SIZE: Size = Size {
        width: 640.0,
        height: 480.0,
    };
    const AABB: AABB = AABB {
        min_x: 0.0,
        max_x: 0.0,
        min_y: -0.0,
        max_y: -0.0,
    };
    let Size { width, height } = AABB.size();
    assert_eq!(0.0, width);
    assert_eq!(0.0, height);
    assert_eq!(SIZE.width / width, f32::INFINITY);
    assert_eq!(SIZE.height / height, f32::INFINITY);
}
