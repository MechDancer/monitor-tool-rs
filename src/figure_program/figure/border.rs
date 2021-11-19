use iced::{canvas::*, Color, Point, Rectangle, Size, Vector};

const BORDER_OFFSET: Point = Point { x: 64.0, y: 32.0 };

#[inline]
pub(super) fn available_size(size: Size) -> Size {
    Size {
        width: size.width - (BORDER_OFFSET.x + 20.0) * 2.0,
        height: size.height - (BORDER_OFFSET.y + 20.0) * 2.0,
    }
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

#[inline]
pub(crate) fn as_available(mut bounds: Rectangle, cursor: Cursor) -> Option<Point> {
    if let Cursor::Available(p) = cursor {
        bounds.x += BORDER_OFFSET.x;
        bounds.y += BORDER_OFFSET.y;
        bounds.width -= BORDER_OFFSET.x * 2.0;
        bounds.height -= BORDER_OFFSET.y * 2.0;
        Some(p).filter(|p| bounds.contains(*p))
    } else {
        None
    }
}

pub(crate) fn mark_cross(bounds: Rectangle, p: Point, rectangle: Rectangle) -> Geometry {
    let size = bounds.size();
    let Size { width, height } = size;
    let mut frame = Frame::new(size);
    let p = p - Vector {
        x: bounds.x,
        y: bounds.y,
    };
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
