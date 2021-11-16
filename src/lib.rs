mod figure;
pub mod figure_canvas;

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
