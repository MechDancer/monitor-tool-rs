use iced::{mouse::Button, Point, Rectangle};

#[derive(Default, Clone, Copy, Debug)]
pub(super) struct Anchor {
    pub bounds: Rectangle,
    pub pos: Point,
    pub which: Option<Button>,
}
