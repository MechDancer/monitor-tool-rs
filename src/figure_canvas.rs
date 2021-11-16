use iced::{canvas::*, Color, Point, Rectangle, Size};

const BORDER_OFFSET: Point = Point { x: 64.0, y: 32.0 };

#[inline]
pub(super) fn available_size(size: Size) -> Size {
    Size {
        width: size.width - (BORDER_OFFSET.x + 20.0) * 2.0,
        height: size.height - (BORDER_OFFSET.y + 20.0) * 2.0,
    }
}

#[inline]
pub(super) fn is_available(size: Size, p: Point) -> bool {
    Rectangle {
        x: BORDER_OFFSET.x,
        y: BORDER_OFFSET.y,
        width: size.width - 2.0 * BORDER_OFFSET.x,
        height: size.height - 2.0 * BORDER_OFFSET.y,
    }
    .contains(p)
}

#[inline]
pub(super) fn border(frame: &mut Frame, color: Color) {
    let Size { width, height } = frame.size();
    frame.stroke(
        &Path::rectangle(
            BORDER_OFFSET,
            Size {
                width: width - BORDER_OFFSET.x * 2.0,
                height: height - BORDER_OFFSET.y * 2.0,
            },
        ),
        Stroke {
            color,
            width: 2.0,
            ..Default::default()
        },
    );
}

pub(super) fn mark_cross(bounds: Size, p: Point, rectangle: Rectangle) -> Geometry {
    let mut frame = Frame::new(bounds);
    let Size { width, height } = frame.size();
    text(
        &mut frame,
        rectangle.x + p.x / width * rectangle.width,
        Point {
            x: p.x + 4.0,
            y: 4.0,
        },
        24.0,
        Color::BLACK,
    );
    text(
        &mut frame,
        rectangle.y - p.y / height * rectangle.height,
        Point {
            x: 4.0,
            y: p.y - 24.0,
        },
        24.0,
        Color::BLACK,
    );
    line(
        &mut frame,
        Point { x: 0.0, y: p.y },
        Point {
            x: p.x - 20.0,
            y: p.y,
        },
        Color::BLACK,
    );
    line(
        &mut frame,
        Point { x: p.x, y: 0.0 },
        Point {
            x: p.x,
            y: p.y - 18.0,
        },
        Color::BLACK,
    );
    line(
        &mut frame,
        Point {
            x: p.x + 20.0,
            y: p.y,
        },
        Point {
            x: width - BORDER_OFFSET.x,
            y: p.y,
        },
        Color::BLACK,
    );
    line(
        &mut frame,
        Point {
            x: p.x,
            y: p.y + 22.0,
        },
        Point {
            x: p.x,
            y: height - BORDER_OFFSET.y,
        },
        Color::BLACK,
    );
    frame.into_geometry()
}

#[inline]
fn line(frame: &mut Frame, p0: Point, p1: Point, color: Color) {
    frame.stroke(
        &Path::line(p0, p1),
        Stroke {
            color,
            width: 1.0,
            ..Default::default()
        },
    );
}

#[inline]
fn text(frame: &mut Frame, num: f32, position: Point, size: f32, color: Color) {
    frame.fill_text(Text {
        content: format!("{:.3}", num),
        position,
        color,
        size,
        ..Default::default()
    });
}
