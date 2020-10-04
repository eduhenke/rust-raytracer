use crate::color::Color;

#[derive(Debug, Copy, Clone)]
pub struct Material {
  pub color: Color,
  pub specular_n: i32,
  pub albedo: f32,
  pub k_diffuse: f32,
  pub k_specular: f32,
}
