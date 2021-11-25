use crate::{Shape, Vertex};
use iced::{Point, Size};

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

    #[inline]
    pub fn contains(&self, p: Point) -> bool {
        self.min_x <= p.x && p.x <= self.max_x && self.min_y <= p.y && p.y <= self.max_y
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
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl std::ops::AddAssign for AABB {
    /// 吸收另一个范围
    fn add_assign(&mut self, rhs: Self) {
        self.absorb(Point {
            x: rhs.min_x,
            y: rhs.min_y,
        });
        self.absorb(Point {
            x: rhs.max_x,
            y: rhs.max_y,
        });
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
