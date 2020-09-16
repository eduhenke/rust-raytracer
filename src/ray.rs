use na::{Vector3, Point3};

#[derive(Debug, Copy, Clone)]
pub struct Ray {
  pub origin: Point3<f32>,
  pub direction: Vector3<f32>,
}
