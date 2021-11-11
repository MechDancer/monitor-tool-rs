﻿use super::{BorderMode, PolarAxis};
use iced::{canvas::*, mouse, Color, Point, Rectangle, Size};

const BORDER_OFFSET: Point = Point { x: 64.0, y: 32.0 };

pub struct Figure {
    border_mode: BorderMode,

    border_cache: Cache,
}

impl Figure {
    pub fn new(border_mode: BorderMode) -> Self {
        Figure {
            border_mode,

            border_cache: Default::default(),
        }
    }
}

impl<Message> Program<Message> for Figure {
    fn draw(&self, bounds: iced::Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(bounds.size());

        let border = self.border_cache.draw(bounds.size(), |frame| {
            let Size {
                width: w,
                height: h,
            } = frame.size();
            let path = match self.border_mode {
                BorderMode::Rectangular => Path::rectangle(
                    BORDER_OFFSET,
                    Size {
                        width: w - BORDER_OFFSET.x * 2.0,
                        height: h - BORDER_OFFSET.y * 2.0,
                    },
                ),
                BorderMode::Polar(_) => Path::circle(
                    Point {
                        x: w / 2.0,
                        y: h / 2.0,
                    },
                    radius(frame.size()),
                ),
            };

            frame.stroke(
                &path,
                Stroke {
                    color: Color::BLACK,
                    width: 2.0,
                    ..Default::default()
                },
            );
        });

        if let Cursor::Available(p) = cursor {
            match self.border_mode {
                BorderMode::Rectangular => mark_cross(&mut frame, p),
                BorderMode::Polar(axis) => mark_polar(&mut frame, p, axis),
            };
        }
        vec![border, frame.into_geometry()]
    }

    fn mouse_interaction(&self, bounds: Rectangle, cursor: Cursor) -> mouse::Interaction {
        if let Cursor::Available(p) = cursor {
            match self.border_mode {
                BorderMode::Rectangular => {
                    if in_bounds_rectangle(bounds.size(), p) {
                        return mouse::Interaction::Crosshair;
                    }
                }
                BorderMode::Polar(_) => {
                    return mouse::Interaction::Crosshair;
                }
            }
        }
        mouse::Interaction::default()
    }
}

fn line(frame: &mut Frame, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
    let line = Path::line(Point { x: x0, y: y0 }, Point { x: x1, y: y1 });
    frame.stroke(
        &line,
        Stroke {
            color,
            width: 1.0,
            ..Default::default()
        },
    );
}

fn text(frame: &mut Frame, num: f32, x: f32, y: f32, size: f32) {
    frame.fill_text(Text {
        content: format!("{:.2}", num),
        position: Point { x, y },
        color: Color::BLACK,
        size,
        ..Default::default()
    });
}

fn radius(size: Size) -> f32 {
    let a = size.width / 2.0 - BORDER_OFFSET.x;
    let b = size.height / 2.0 - BORDER_OFFSET.y;
    if a < b {
        a
    } else {
        b
    }
}

fn in_bounds_rectangle(size: Size, p: Point) -> bool {
    Rectangle {
        x: BORDER_OFFSET.x,
        y: BORDER_OFFSET.y,
        width: size.width - 2.0 * BORDER_OFFSET.x,
        height: size.height - 2.0 * BORDER_OFFSET.y,
    }
    .contains(p)
}

fn mark_cross(frame: &mut Frame, p: Point) {
    let Size {
        width: w,
        height: h,
    } = frame.size();
    if in_bounds_rectangle(frame.size(), p) {
        text(frame, p.x, p.x + 4.0, 4.0, 24.0);
        text(frame, p.y, 4.0, p.y - 24.0, 24.0);
        line(frame, 0.0, p.y, p.x - 20.0, p.y, Color::BLACK);
        line(frame, p.x, 0.0, p.x, p.y - 18.0, Color::BLACK);
        line(
            frame,
            p.x + 20.0,
            p.y,
            w - BORDER_OFFSET.x,
            p.y,
            Color::BLACK,
        );
        line(
            frame,
            p.x,
            p.y + 22.0,
            p.x,
            h - BORDER_OFFSET.y,
            Color::BLACK,
        );
    }
}

fn mark_polar(frame: &mut Frame, p: Point, axis: PolarAxis) {
    use std::f32::consts::PI;

    let Size {
        width: w,
        height: h,
    } = frame.size();
    let p = na::Vector2::new(p.x, p.y);
    let c = na::Vector2::new(w / 2.0, h / 2.0);
    let d = p - c;
    let u = d.normalize();
    let r = radius(frame.size());
    let v = u * r + c;

    line(frame, 0.0, c.y, w, c.y, Color::BLACK);
    line(frame, c.x, 0.0, c.x, h, Color::BLACK);
    line(frame, c.x, c.y, v.x, v.y, Color::BLACK);

    let degree = match axis {
        PolarAxis::Top => d.x.atan2(-d.y),
        PolarAxis::Left => d.y.atan2(d.x),
    } * (-180.0 / PI);
    let text = Text {
        content: if degree < -0.01 {
            format!("-{:.2}°", -degree)
        } else {
            format!(" {:.2}°", degree.abs())
        },
        position: Point { x: v.x, y: v.y },
        color: Color::BLACK,
        size: 24.0,
        ..Default::default()
    };
    if d.y > 0.0 {
        if d.x > 0.0 {
            frame.fill_text(Text {
                horizontal_alignment: iced::HorizontalAlignment::Left,
                vertical_alignment: iced::VerticalAlignment::Top,
                ..text
            });
        } else {
            frame.fill_text(Text {
                horizontal_alignment: iced::HorizontalAlignment::Right,
                vertical_alignment: iced::VerticalAlignment::Top,
                ..text
            });
        }
    } else {
        if d.x > 0.0 {
            frame.fill_text(Text {
                horizontal_alignment: iced::HorizontalAlignment::Left,
                vertical_alignment: iced::VerticalAlignment::Bottom,
                ..text
            });
        } else {
            frame.fill_text(Text {
                horizontal_alignment: iced::HorizontalAlignment::Right,
                vertical_alignment: iced::VerticalAlignment::Bottom,
                ..text
            });
        }
    }
    let rho = d.norm();
    if rho < r {
        let text = Text {
            content: format!("{:.2}", rho),
            position: Point { x: p.x, y: p.y },
            color: Color::BLACK,
            size: 24.0,
            ..Default::default()
        };
        if d.y > 0.0 {
            if d.x > 0.0 {
                frame.fill_text(Text {
                    horizontal_alignment: iced::HorizontalAlignment::Left,
                    vertical_alignment: iced::VerticalAlignment::Bottom,
                    ..text
                });
            } else {
                frame.fill_text(Text {
                    horizontal_alignment: iced::HorizontalAlignment::Right,
                    vertical_alignment: iced::VerticalAlignment::Bottom,
                    ..text
                });
            }
        } else {
            if d.x > 0.0 {
                frame.fill_text(Text {
                    horizontal_alignment: iced::HorizontalAlignment::Left,
                    vertical_alignment: iced::VerticalAlignment::Top,
                    ..text
                });
            } else {
                frame.fill_text(Text {
                    horizontal_alignment: iced::HorizontalAlignment::Right,
                    vertical_alignment: iced::VerticalAlignment::Top,
                    ..text
                });
            }
        }
    }
}
