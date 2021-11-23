use iced::{mouse::Button, Point};

#[derive(Default, Clone, Copy, Debug)]
pub(super) struct Anchor {
    pub pos: Point,
    pub which: Option<Button>,
}
