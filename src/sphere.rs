use na::{Unit, Point3};
use sdl2::pixels::Color;
use super::ray::Ray;


#[derive(Debug)]
pub struct Sphere {
  pub center: Point3<f32>,
  pub radius: f32
}

// simplified to either have 0 roots or 2(instead of 0, 1, 2)
fn find_roots_quadratic (a: f32, b: f32, c: f32) -> Option<(f32, f32)> {
  let discriminant = b*b - 4.*a*c;
  if discriminant < 0. {
      None
  } else {
      let sq = discriminant.sqrt();
      Some(((-b-sq)/(2.*a), (-b+sq)/(2.*a)))
  }
}

impl Sphere {
  // https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection
  fn find_roots_intersection(&self, Ray { origin, direction }: &Ray) -> Option<(f32, f32)> {
      // println!("{:?}", Ray{origin: *origin, direction: *direction});
      let diff = origin - self.center;
      // println!("diff: {:?}", diff);
      // println!("center: {:?}", self.center);
      let a = direction.dot(direction);
      let b = 2. * direction.dot(&diff); 
      let c = diff.norm_squared() - self.radius.powi(2);

      find_roots_quadratic(a, b, c)
  }
  pub fn get_color(&self, ray: &Ray, light: &Ray) -> Color {
      match self.find_roots_intersection(ray) {
          None => Color::BLACK,
          Some((t0, t1)) => {
              let mut t = t0;
              if t0 > t1 {
                  t = t1;
              }
              if t0 < 0. {
                  if t1 < 0. { // both t0 and t1 are negative
                      return Color::BLACK
                  }
                  t = t1;
              }
              // println!("t0: {} t1: {}", t0, t1);
              let point_hit = ray.origin + (t*ray.direction);

              let normal = Unit::new_normalize(point_hit - self.center);
              
              let pointing_to_light = Unit::new_normalize(light.origin - point_hit);
              let lightness_percentage: f32 = normal.into_inner().dot(&pointing_to_light);
              
              let color = (255.*lightness_percentage.max(0.)) as u8;
              Color::RGB(color, color, color)
          }
      }
  }
}