use super::ray::Ray;
use na::{Point3, Unit, Vector3};
use std::fmt::Debug;
pub mod plane;
pub mod sphere;

pub struct CastInfo {
  pub normal: Unit<Vector3<f32>>,
  pub point_hit: Point3<f32>,
  pub distance: f32,
}

pub trait Castable {
  fn cast_ray(&self, ray: &Ray) -> Option<CastInfo>;
}

pub trait Movable {
  fn move_to(&mut self, direction: Vector3<f32>);
}

pub trait Shape: Castable + Movable + Debug {}
