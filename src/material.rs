use crate::color::Color;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MaterialType {
  Reflection {
    reflectivity: f32,
  },
  Refraction {
    refractive_index: f32,
  },
  Phong {
    k_specular: f32,
    k_diffuse: f32,
    specular_n: i32,
  },
}

#[derive(Debug, Copy, Clone)]
pub struct Material {
  pub color: Color,
  pub albedo: f32,
  pub material_type: MaterialType,
}
