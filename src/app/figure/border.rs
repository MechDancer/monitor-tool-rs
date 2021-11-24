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
pub(super) fn border(frame: &mut Frame, backbround: Color, foreground: Color) {
    let size = frame.size();
    if backbround != Color::WHITE {
        frame.fill(&Path::rectangle(Point::ORIGIN, size), backbround);
    }
    frame.stroke(
        &Path::rectangle(
            BORDER_OFFSET,
            Size {
                width: size.width - BORDER_OFFSET.x * 2.0,
                height: size.height - BORDER_OFFSET.y * 2.0,
            },
        ),
        Stroke {
            color: foreground,
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

macro_rules! line {
    ($frame:expr; ($x0:expr, $y0:expr) => ($x1:expr, $y1:expr); $color:expr) => {
        $frame.stroke(
            &Path::line(Point { x: $x0, y: $y0 }, Point { x: $x1, y: $y1 }),
            Stroke {
                color: $color,
                width: 1.0,
                ..Default::default()
            },
        )
    };
}

pub(crate) fn mark_cross(
    frame: &mut Frame,
    bounds: Rectangle,
    p: Point,
    rectangle: Rectangle,
    color: Color,
) {
    let size = bounds.size();
    let Size { width, height } = size;
    let p = p - Vector {
        x: bounds.x,
        y: bounds.y,
    };
    text(
        frame,
        rectangle.x + p.x / width * rectangle.width,
        Point {
            x: p.x + 4.0,
            y: 4.0,
        },
        24.0,
        color,
    );
    text(
        frame,
        rectangle.y - p.y / height * rectangle.height,
        Point {
            x: 4.0,
            y: p.y - 24.0,
        },
        24.0,
        color,
    );
    line!(frame; (0.0, p.y) => (p.x - 20.0, p.y       ); color);
    line!(frame; (p.x, 0.0) => (p.x       , p.y - 18.0); color);
    line!(frame; (p.x + 20.0, p.y       ) => (width - BORDER_OFFSET.x,  p.y); color);
    line!(frame; (p.x       , p.y + 22.0) => (p.x, height - BORDER_OFFSET.y); color);
}

pub(crate) fn mark_anchor(frame: &mut Frame, p0: Point, p1: Point, color: Color) {
    line!(frame; (p0.x, p0.y) => (p0.x, p1.y); color);
    line!(frame; (p0.x, p0.y) => (p1.x, p0.y); color);
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
