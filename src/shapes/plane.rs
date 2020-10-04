use super::Castable;
use super::{super::ray::Ray, CastInfo};
use crate::shapes::Movable;
use crate::shapes::Shape;
use na::{Isometry3, Point3, Unit, Vector3};

#[derive(Debug, Copy, Clone)]
pub struct Plane {
  pub normal: Unit<Vector3<f32>>,
  pub center: Point3<f32>,
}

impl Castable for Plane {
  // https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-plane-and-ray-disk-intersection
  fn cast_ray(&self, ray: &Ray) -> Option<CastInfo> {
    let denominator = self.normal.into_inner().dot(&ray.direction);
    if denominator > 0. {
      return None;
    }
    let t = (self.center - ray.origin).dot(&self.normal.into_inner()) / denominator;
    if t < 0. {
      return None;
    }

    let point_hit = ray.origin + (t * ray.direction.into_inner());

    Some(CastInfo {
      distance: t,
      normal: self.normal,
      pointing_to_viewer: Unit::new_normalize(ray.origin - point_hit),
      point_hit,
      albedo: 1.,
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
