use super::{super::ray::Ray, CastInfo};
use super::{Castable, Movable};
use crate::{material::Material, shapes::Shape};
use na::{Isometry3, Point3, Unit};
use nalgebra::Vector3;

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
  center: Point3<f32>,
  radius: f32,
  material: Material,

  world_to_object: Isometry3<f32>,
  object_to_world: Isometry3<f32>,
}

impl Sphere {
  pub fn new(center: Point3<f32>, radius: f32, material: Material) -> Sphere {
    let model_matrix = Isometry3::new(center.coords, na::zero());
    Sphere {
      center: Point3::new(0., 0., 0.),
      radius: radius,
      material: material,
      object_to_world: model_matrix,
      world_to_object: model_matrix.inverse(),
    }
  }
}

// simplified to either have 0 roots or 2(instead of 0, 1, 2)
fn find_roots_quadratic(a: f32, b: f32, c: f32) -> Option<(f32, f32)> {
  let discriminant = b * b - 4. * a * c;
  if discriminant < 0. {
    None
  } else {
    let sq = discriminant.sqrt();
    Some(((-b - sq) / (2. * a), (-b + sq) / (2. * a)))
  }
}

impl Sphere {
  // https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection
  fn find_roots_intersection(&self, Ray { origin, direction }: &Ray) -> Option<(f32, f32)> {
    let diff = origin - self.center;
    let a = direction.dot(direction);
    let b = 2. * direction.dot(&diff);
    let c = diff.norm_squared() - self.radius.powi(2);

    find_roots_quadratic(a, b, c)
  }
}

impl Castable for Sphere {
  fn cast_ray(&self, world_ray: &Ray) -> Option<CastInfo> {
    let ray = &world_ray.apply_isometry(self.world_to_object);
    let a = self.find_roots_intersection(ray);
    match a {
      None => None,
      Some((t0, t1)) => {
        let mut t = t0;
        if t0 > t1 {
          t = t1;
        }
        if t0 < 0. {
          if t1 < 0. {
            // both t0 and t1 are negative
            return None;
          }
          t = t1;
        }
        let point_hit = ray.origin + (t * ray.direction.into_inner());

        let normal = Unit::new_normalize(point_hit - self.center);

        Some(
          CastInfo {
            normal: normal,
            point_hit: point_hit,
            pointing_to_viewer: Unit::new_normalize(ray.origin - point_hit),
            distance: t,
            casted: self,
            material: self.material,
          }
          .apply_isometry(self.object_to_world),
        )
      }
    }
  }
}

impl Movable for Sphere {
  fn move_to(&mut self, direction: Vector3<f32>) {
    self.center += direction;
  }
}

impl Shape for Sphere {}
