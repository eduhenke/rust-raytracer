use na::{Isometry3, Point3, Unit, Vector3};

#[derive(Debug, Copy, Clone)]
pub struct Ray {
  pub origin: Point3<f32>,
  pub direction: Unit<Vector3<f32>>,
}

impl Ray {
  pub fn apply_isometry(&self, isometry: Isometry3<f32>) -> Self {
    Self {
      origin: isometry.transform_point(&self.origin),
      direction: Unit::new_unchecked(isometry.transform_vector(&self.direction)),
    }
  }
}
