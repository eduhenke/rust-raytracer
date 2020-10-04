use super::ray::Ray;
use na::{Isometry3, Point3, Unit, Vector3};
use std::fmt::Debug;
pub mod plane;
pub mod sphere;

pub struct CastInfo<'a> {
  pub normal: Unit<Vector3<f32>>,
  pub pointing_to_viewer: Unit<Vector3<f32>>,
  pub point_hit: Point3<f32>,
  pub distance: f32,
  pub casted: &'a dyn Castable,
}

impl<'a> CastInfo<'a> {
  pub fn apply_isometry(&self, isometry: Isometry3<f32>) -> CastInfo<'a> {
    CastInfo {
      normal: Unit::new_unchecked(isometry.transform_vector(&self.normal.into_inner())),
      point_hit: isometry.transform_point(&self.point_hit),
      pointing_to_viewer: Unit::new_unchecked(isometry.transform_vector(&self.pointing_to_viewer)),
      distance: self.distance,
      casted: self.casted,
    }
  }
}

pub fn get_nearest_cast_info<'a>(
  a: Option<CastInfo<'a>>,
  b: Option<CastInfo<'a>>,
) -> Option<CastInfo<'a>> {
  match a {
    None => b,
    Some(a_info) => match b {
      None => Some(a_info),
      Some(b_info) => {
        if a_info.distance > b_info.distance {
          return Some(b_info);
        } else {
          return Some(a_info);
        }
      }
    },
  }
}

pub trait Castable {
  fn cast_ray(&self, ray: &Ray) -> Option<CastInfo>;
  fn albedo(&self) -> f32;
  fn specular_n(&self) -> i32;
}

pub trait Movable {
  fn move_to(&mut self, direction: Vector3<f32>);
}

pub trait Shape: Castable + Movable + Debug {
  fn model_matrix(&self) -> Isometry3<f32>;
  fn inverse_model_matrix(&self) -> Isometry3<f32>;
}
