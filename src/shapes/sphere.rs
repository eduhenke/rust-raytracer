use super::{super::ray::Ray, CastInfo};
use super::{Castable, Movable};
use crate::shapes::Shape;
use na::{Point3, Unit};
use nalgebra::Vector3;
use sdl2::pixels::Color;

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
  pub center: Point3<f32>,
  pub radius: f32,
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
  fn cast_ray(&self, ray: &Ray) -> Option<CastInfo> {
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
        let point_hit = ray.origin + (t * ray.direction);

        let normal = Unit::new_normalize(point_hit - self.center);

        Some(CastInfo {
          normal: normal,
          point_hit: point_hit,
          distance: t,
        })
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
