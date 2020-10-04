use super::Castable;
use super::{super::ray::Ray, CastInfo};
use crate::shapes::Shape;
use crate::{material::Material, shapes::Movable};
use na::{Isometry3, Point3, Unit, Vector3};

#[derive(Debug, Copy, Clone)]
pub struct Plane {
  normal: Unit<Vector3<f32>>,
  center: Point3<f32>,
  size: (Option<f32>, Option<f32>),
  material: Material,

  world_to_object: Isometry3<f32>,
  object_to_world: Isometry3<f32>,
}

impl Plane {
  pub fn new(
    normal: Unit<Vector3<f32>>,
    center: Point3<f32>,
    size: (Option<f32>, Option<f32>),
    rotation: Vector3<f32>,
    material: Material,
  ) -> Plane {
    let model_matrix = Isometry3::new(center.coords, rotation);
    Plane {
      center: Point3::new(0., 0., 0.),
      object_to_world: model_matrix,
      world_to_object: model_matrix.inverse(),
      normal: normal,
      size: size,
      material: material,
    }
  }
}

impl Castable for Plane {
  // https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-plane-and-ray-disk-intersection
  fn cast_ray(&self, world_ray: &Ray) -> Option<CastInfo> {
    let ray = world_ray.apply_isometry(self.world_to_object);
    let denominator = self.normal.into_inner().dot(&ray.direction);
    if denominator > 0. {
      return None;
    }
    let t = (self.center - ray.origin).dot(&self.normal.into_inner()) / denominator;
    if t < 0. {
      return None;
    }

    let point_hit = ray.origin + (t * ray.direction.into_inner());
    let distance_to_center = point_hit.to_homogeneous() - self.center.to_homogeneous();
    match self.size.0 {
      Some(x_) => {
        let x = distance_to_center.x;
        if x.abs() > x_ {
          return None;
        }
      }
      None => {}
    }
    match self.size.1 {
      Some(z_) => {
        let z = distance_to_center.z;
        if z.abs() > z_ {
          return None;
        }
      }
      None => {}
    }
    Some(
      CastInfo {
        distance: t,
        normal: self.normal,
        pointing_to_viewer: Unit::new_normalize(ray.origin - point_hit),
        point_hit,
        casted: self,
        material: self.material,
      }
      .apply_isometry(self.object_to_world),
    )
  }
}

impl Movable for Plane {
  fn move_to(&mut self, _direction: Vector3<f32>) {
    todo!()
  }
}

impl Shape for Plane {}
