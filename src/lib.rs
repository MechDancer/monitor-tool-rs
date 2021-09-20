pub mod cache2d;
pub mod canvas2d;
pub mod protocol;

#[derive(Clone, Copy)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
    pub theta: f32,
}
