use na::{Unit, Vector3};

pub fn reflect(v: &Vector3<f32>, n: &Unit<Vector3<f32>>) -> Vector3<f32> {
  2.0 * n.dot(v) * n.into_inner() - v
}

pub fn refract(
  i: &Unit<Vector3<f32>>,
  n: &Unit<Vector3<f32>>,
  n_i: f32,
  n_t: f32,
) -> Option<Vector3<f32>> {
  let cos_i = -n.dot(i).max(-1.0).min(1.0);
  let theta_i: f32 = cos_i.acos();
  // println!("theta_i: {} | cos_i: {}", theta_i.to_degrees(), cos_i);
  if theta_i.is_nan() {
    return None;
  }
  let sin_i = theta_i.sin();

  // snell's law
  let sin_t = (n_i / n_t) * sin_i;
  let theta_t = sin_t.asin();
  if theta_t.is_nan() {
    return None;
  }
  let cos_t = theta_t.cos();

  let tangent = {
    if sin_i > 0. {
      ((n.as_ref() * cos_i) + i.as_ref()) / sin_i
    } else {
      na::zero()
    }
  };
  let a = sin_t * tangent;
  let b = cos_t * -n.as_ref();
  let t: Vector3<f32> = a + b;
  Some(t)
}

pub fn fresnel(i: &Unit<Vector3<f32>>, n: &Unit<Vector3<f32>>, n_i: f32, n_t: f32) -> (f32, f32) {
  let cos_i = -n.dot(i).max(-1.0).min(1.0);
  let theta_i: f32 = cos_i.acos();
  let sin_i = theta_i.sin();

  // snell's law
  let sin_t = (n_i / n_t) * sin_i;
  if sin_t >= 1. {
    return (1., 0.);
  }
  let theta_t = sin_t.asin();
  let cos_t = theta_t.cos();
  let rs = ((n_t * cos_i) - (n_i * cos_t)) / ((n_t * cos_i) + (n_i * cos_t));
  let rp = ((n_i * cos_t) - (n_t * cos_i)) / ((n_i * cos_t) + (n_t * cos_i));
  let kr = (rs.powi(2) + rp.powi(2)) / 2.;
  (kr, 1. - kr)
}

#[cfg(test)]
mod tests {
  use nalgebra::Isometry3;

  use super::*;

  #[test]
  fn test_reflect() {
    let n = Unit::new_normalize(Vector3::new(0., 1., 0.));
    let v = Vector3::new(-1., -1., 0.);
    assert_eq!(reflect(&v, &n), Vector3::new(1., -1., 0.));
  }

  macro_rules! assert_vec_float_eq {
    ($x:expr, $y:expr) => {
      if $x.x.is_nan() {
        panic!("no component can be NaN");
      }
      let difference: Vector3<f32> = $x - $y;
      if difference.norm() > 0.001 {
        panic!("vector {:?} do not equal {:?}", $x, $y);
      };
    };
  }

  fn vec_from_angle(degrees: f32) -> Vector3<f32> {
    let (sin, cos) = degrees.to_radians().sin_cos();
    Vector3::new(cos, sin, 0.)
  }

  fn unit_apply_isometry(u: &Unit<Vector3<f32>>, isometry: &Isometry3<f32>) -> Unit<Vector3<f32>> {
    Unit::new_unchecked(isometry.transform_vector(u.as_ref()))
  }

  #[test]
  fn test_refract() {
    let n = Unit::new_normalize(Vector3::new(0., 1., 0.));
    let v = Unit::new_normalize(vec_from_angle(-45.));
    let isometry = Isometry3::new(Vector3::new(10., -20., -5.), Vector3::y() * 0.5);

    // test normal refract and with isometry applied
    let test_normal_and_isometry = |i: &Unit<Vector3<f32>>,
                                    n: &Unit<Vector3<f32>>,
                                    n_i: f32,
                                    n_t: f32,
                                    expected: Vector3<f32>| {
      assert_vec_float_eq!(refract(&i, &n, n_i, n_t).unwrap(), expected);
      assert_vec_float_eq!(
        refract(
          &unit_apply_isometry(&i, &isometry),
          &unit_apply_isometry(&n, &isometry),
          n_i,
          n_t
        )
        .unwrap(),
        &unit_apply_isometry(&Unit::new_normalize(expected), &isometry).into_inner()
      );
    };

    test_normal_and_isometry(&v, &n, 1., 1., v.into_inner());

    // works with reflection
    let v_r = Unit::new_unchecked(reflect(&v, &n));
    test_normal_and_isometry(&v_r, &n, 1., 1., v_r.into_inner());

    let v_60 = Unit::new_normalize(vec_from_angle(-60.));
    test_normal_and_isometry(&v_60, &n, 1., 1., v_60.into_inner());

    let refracted = vec_from_angle(-90. + 28.1255);
    // to higher refractive index
    test_normal_and_isometry(&v, &n, 1., 1.5, refracted);

    // to lower refractive index
    test_normal_and_isometry(&Unit::new_normalize(refracted), &n, 1.5, 1., v.into_inner());

    // total internal reflection
    assert_eq!(refract(&v, &n, 1.5, 1.), None);

    // (n_i, n_t, bias): (1.0, 1.0, Matrix { data: [-0.0, -0.0, 0.1] }) | outside: true | n: Unit { value: Matrix { data: [0.0, 0.0, -1.0] } } | dir: Unit { value: Matrix { data: [-0.0, -0.0, 1.0] } }
    test_normal_and_isometry(
      &Unit::new_normalize(Vector3::new(-0., -0., 1.)),
      &Unit::new_normalize(Vector3::new(0., 0., -1.)),
      1.,
      1.,
      Vector3::new(-0., -0., 1.),
    );
  }
}
