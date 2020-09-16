use super::Castable;
use super::{super::ray::Ray, CastInfo};
use crate::shapes::Movable;
use crate::shapes::Shape;
use na::{Point3, Unit, Vector3};

#[derive(Debug, Copy, Clone)]
pub struct Plane {
  pub normal: Unit<Vector3<f32>>,
}

impl Castable for Plane {
  fn cast_ray(&self, ray: &Ray) -> Option<CastInfo> {
    if self.normal.into_inner().dot(&ray.direction) > 0. {
      return None;
    }
    Some(CastInfo {
      distance: 5e9,
      normal: self.normal,
      point_hit: Point3::new(0., 0., 0.),
    })
  }
}

impl Movable for Plane {
  fn move_to(&mut self, direction: Vector3<f32>) {
    todo!()
  }
}

impl Shape for Plane {}