pub mod canvas2d;
mod figure;

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
