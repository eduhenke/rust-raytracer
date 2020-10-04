use crate::color::Color;
use crate::ray::Ray;

#[derive(Debug, Copy, Clone)]
pub struct PointLight {
  pub ray: Ray,
  pub color: Color,
  pub intensity: f32,
}
