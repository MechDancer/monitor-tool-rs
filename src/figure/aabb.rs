﻿use iced::{Point, Rectangle, Size};

#[derive(PartialEq, Clone, Copy)]
pub struct AABB {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

impl From<Point> for AABB {
    fn from(p: Point) -> Self {
        Self {
            min_x: p.x,
            max_x: p.x,
            min_y: p.y,
            max_y: p.y,
        }
    }
}

impl AABB {
    pub fn foreach(mut iter: impl Iterator<Item = Point>) -> Option<Self> {
        iter.next().map(|front| {
            let mut aabb = AABB::from(front);
            iter.for_each(|p| aabb.absorb(p));
            aabb
        })
    }

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

    pub fn to_rectangle(&self) -> Rectangle {
        Rectangle::new(
            Point {
                x: self.min_x,
                y: self.min_y,
            },
            Size::new(self.max_x - self.min_x, self.max_y - self.min_y),
        )
    }
}

impl std::ops::Add for AABB {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.absorb(Point {
            x: rhs.min_x,
            y: rhs.min_y,
        });
        self.absorb(Point {
            x: rhs.max_x,
            y: rhs.max_y,
        });
        self
    }
}
