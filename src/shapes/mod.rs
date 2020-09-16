use super::ray::Ray;
use na::{Point3, Unit, Vector3};
pub mod sphere;

pub struct CastInfo {
  pub normal: Unit<Vector3<f32>>,
  pub point_hit: Point3<f32>,
}

pub trait Castable {
  fn cast_ray(&self, ray: &Ray) -> Option<CastInfo>;
}
