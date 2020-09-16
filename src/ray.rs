use na::{Point3, Unit, Vector3};

#[derive(Debug, Copy, Clone)]
pub struct Ray {
  pub origin: Point3<f32>,
  pub direction: Unit<Vector3<f32>>,
}
