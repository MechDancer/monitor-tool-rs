extern crate nalgebra as na;

pub mod canvas2d;
pub mod protocol;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BorderMode {
    Rectangular,
    Polar(PolarAxis),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PolarAxis {
    Top,
    Left,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
    pub theta: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
