use super::ray::Ray;
use na::{Isometry3, Point3, Unit, Vector3};
use std::fmt::Debug;
pub mod plane;
pub mod sphere;

pub struct CastInfo {
  pub normal: Unit<Vector3<f32>>,
  pub pointing_to_viewer: Unit<Vector3<f32>>,
  pub point_hit: Point3<f32>,
  pub distance: f32,
}

pub trait Castable {
  fn cast_ray(&self, ray: &Ray) -> Option<CastInfo>;
}

pub trait Movable {
  fn move_to(&mut self, direction: Vector3<f32>);
}

pub trait Shape: Castable + Movable + Debug {
  fn model_matrix(&self) -> Isometry3<f32>;
  fn inverse_model_matrix(&self) -> Isometry3<f32>;
}
