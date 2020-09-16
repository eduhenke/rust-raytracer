use na::{Point3, Vector3};

#[derive(Debug, Copy, Clone)]
pub struct Ray {
  pub origin: Point3<f32>,
  pub direction: Vector3<f32>,
}
