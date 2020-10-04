use sdl2::pixels::Color as SdlColor;
use std::ops::{Add, AddAssign, Mul};

#[derive(Debug, Copy, Clone)]
pub struct Color {
  r: u8,
  g: u8,
  b: u8,
}

impl Color {
  #[allow(non_snake_case)]
  pub const fn RGB(r: u8, g: u8, b: u8) -> Color {
    Color { r, g, b }
  }
}

impl Mul<f32> for Color {
  type Output = Self;

  fn mul(self, rhs: f32) -> Self {
    let times = |x| ((x as f32) * rhs).min(255.) as u8;
    Color::RGB(times(self.r), times(self.g), times(self.b))
  }
}

impl Add for Color {
  type Output = Self;
  fn add(self, rhs: Self) -> Self {
    let plus = |a: u8, b: u8| ((a as u16) + (b as u16)).min(255).max(0) as u8;
    Color::RGB(
      plus(self.r, rhs.r),
      plus(self.g, rhs.g),
      plus(self.b, rhs.b),
    )
  }
}

impl AddAssign for Color {
  fn add_assign(&mut self, other: Self) {
    *self = *self + other
  }
}

impl Into<SdlColor> for Color {
  fn into(self) -> SdlColor {
    SdlColor::RGB(self.r, self.g, self.b)
  }
}

impl From<SdlColor> for Color {
  fn from(raw: SdlColor) -> Color {
    Color::RGB(raw.r, raw.g, raw.b)
  }
}
