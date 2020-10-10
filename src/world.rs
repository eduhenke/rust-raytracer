use crate::shapes::Shape;
use crate::{color::Color, light::PointLight};
use crate::{
  ops::{reflect, refract},
  shapes::CastInfo,
};
use crate::{ray::Ray, shapes::get_nearest_cast_info};
use na::Unit;
use std::f32::consts::PI;

#[derive(Debug)]
pub struct World<'a> {
  pub shapes: Vec<&'a (dyn Shape + Sync)>,
  pub lights: Vec<PointLight>,
}

const BACKGROUND: Color = Color::RGB(50, 0, 50);
const MAX_RAY_DEPTH: i32 = 3;

impl<'a> World<'a> {
  fn get_lighting(&self, info: &CastInfo, light: &PointLight) -> (Color, Color) {
    let nudge = info.normal.into_inner() * 0.01;

    // https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-shading/shading-normals
    let distance_to_light = light.ray.origin - info.point_hit;
    let pointing_to_light = Unit::new_normalize(distance_to_light);
    let point_to_light_crosses_object = self.cast_to_shapes(&Ray {
      origin: info.point_hit + nudge,
      direction: pointing_to_light,
    });
    match point_to_light_crosses_object {
      Some(shadow_info) => {
        // only shadow if casted object is nearer than light
        if shadow_info.distance < distance_to_light.norm() {
          return (Color::zero(), Color::zero());
        }
      }
      None => {}
    }
    let facing_ratio: f32 = info.normal.dot(&pointing_to_light).max(0.);
    let reflected_light = reflect(&pointing_to_light.into_inner(), &info.normal);

    // intensity at point hit
    let light_intensity_at_point = light.intensity / (4. * PI * distance_to_light.norm_squared());
    let diffuse = light.color * (info.material.albedo * light_intensity_at_point * facing_ratio);
    let specular = light.color
      * light_intensity_at_point
      * info
        .pointing_to_viewer
        .into_inner()
        .dot(&reflected_light)
        .max(0.)
        .powi(info.material.specular_n);
    (diffuse, specular)
  }
  fn get_reflected_ray(&self, info: &CastInfo) -> Ray {
    let nudge = info.normal.into_inner() * 0.01;

    let reflection = reflect(&info.pointing_to_viewer, &info.normal);
    Ray {
      direction: Unit::new_normalize(reflection),
      origin: info.point_hit + nudge,
    }
  }
  fn get_refracted_ray(&self, info: &CastInfo) -> Ray {
    let ray_direction = -info.pointing_to_viewer;
    let ior = info.material.index_of_refraction.unwrap();
    let nudge = info.normal.into_inner() * 0.1;

    let outside = info.normal.dot(&ray_direction) < 0.;
    let (n_i, n_t, bias) = {
      if outside {
        (1., ior, -nudge)
      } else {
        (ior, 1., nudge)
      }
    };
    // println!(
    //   "(n_i, n_t, bias): {:?} | outside: {} | n: {:?} | dir: {:?}",
    //   (n_i, n_t, bias),
    //   outside,
    //   info.normal,
    //   ray_direction
    // );
    match refract(&ray_direction, &info.normal, n_i, n_t) {
      Some(refraction) => {
        // println!("{:?}", refraction);
        Ray {
          direction: Unit::new_normalize(refraction),
          origin: info.point_hit + bias,
        }
      }
      // total internal reflection
      None => self.get_reflected_ray(&info),
    }
  }
  pub fn cast_to_shapes(&self, ray: &Ray) -> Option<CastInfo> {
    self
      .shapes
      .iter()
      .map(|obj| obj.cast_ray(&ray))
      .fold(None, get_nearest_cast_info)
  }
  pub fn trace(&self, ray: &Ray) -> Option<(CastInfo, Ray)> {
    match self.cast_to_shapes(ray) {
      None => None,
      Some(info) => match info.material.index_of_refraction {
        Some(_) => Some((info, self.get_refracted_ray(&info))),
        None => Some((info, self.get_reflected_ray(&info))),
      },
    }
  }
  pub fn get_color_at_ray(&self, ray: &Ray) -> Color {
    let mut trace_result: Option<(CastInfo, Ray)> = self.trace(ray);
    let mut specular_path: Vec<CastInfo> = vec![];
    let mut color = Color::zero();
    for _ in 0..MAX_RAY_DEPTH {
      match trace_result {
        None => {}
        Some((info, ray)) => {
          specular_path.push(info);
          trace_result = self.trace(&ray);
        }
      }
    }
    let mut reflectivity = 1.;
    for object in specular_path.iter() {
      // Multiple lights(https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-shading/shading-multiple-lights)
      let (diffuse, specular) = self
        .lights
        .iter()
        .map(|light| self.get_lighting(&object, light))
        .fold((Color::zero(), Color::zero()), |(a1, a2), (b1, b2)| {
          (a1 + b1, a2 + b2)
        });

      let object_color =
        diffuse * object.material.k_diffuse + specular * object.material.k_specular;

      color += object_color * reflectivity;
      reflectivity *= object.material.k_specular;
    }
    color
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{material::Material, shapes::sphere::Sphere};
  use nalgebra::{Point3, Vector3};

  #[test]
  fn test_refraction() {
    let sphere = Sphere::new(
      Point3::new(0., 0., 0.),
      1.,
      Material {
        color: Color::RGB(0, 0, 0),
        albedo: 1.0,
        specular_n: 1,
        k_diffuse: 0.0,
        k_specular: 1.0,
        index_of_refraction: Some(1.0),
      },
    );
    let world = World {
      shapes: vec![&sphere],
      lights: vec![],
    };
    let looking_at_sphere = &Ray {
      origin: Point3::new(0., 0., -10.),
      direction: Unit::new_normalize(Vector3::new(0., 0., 1.)),
    };

    // assert_eq!(world.get_color_at_ray(looking_at_sphere, 0), BACKGROUND);

    // let first_cast = world.trace(looking_at_sphere).unwrap();
    // assert_eq!(first_cast.point_hit, Point3::new(0., 0., -1.));

    // let refracted_ray = &world.get_refracted_ray(&first_cast);
    // println!("{:?}", refracted_ray);
    // let second_cast = world.trace(refracted_ray).unwrap();
    // assert_eq!(second_cast.point_hit, Point3::new(0., 0., 1.));
    // world.
  }
}
