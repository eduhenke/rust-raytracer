use super::Castable;
use super::{super::ray::Ray, CastInfo};
use crate::shapes::Movable;
use crate::shapes::Shape;
use na::{Isometry3, Point3, Unit, Vector3};

#[derive(Debug, Copy, Clone)]
pub struct Plane {
  pub normal: Unit<Vector3<f32>>,
}

impl Castable for Plane {
  fn cast_ray(&self, ray: &Ray) -> Option<CastInfo> {
    if self.normal.into_inner().dot(&ray.direction) > 0. {
      return None;
    }
    let point_hit = Point3::new(0., 0., 0.);
    Some(CastInfo {
      distance: 5e9,
      normal: self.normal,
      pointing_to_viewer: Unit::new_normalize(ray.origin - point_hit),
      point_hit,
    })
  }
}

impl Movable for Plane {
  fn move_to(&mut self, _direction: Vector3<f32>) {
    todo!()
  }
}

impl Shape for Plane {
  fn model_matrix(&self) -> Isometry3<f32> {
    Isometry3::new(na::zero(), na::zero())
  }
  fn inverse_model_matrix(&self) -> Isometry3<f32> {
    Isometry3::new(na::zero(), na::zero())
  }
}
