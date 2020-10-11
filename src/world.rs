use crate::ops::fresnel;
use crate::{color::Color, light::PointLight};
use crate::{material::MaterialType, shapes::Shape};
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

const BACKGROUND: Color = Color::RGB(59, 172, 214);
const MAX_RAY_DEPTH: i32 = 10;

impl<'a> World<'a> {
  fn get_lighting(&self, info: &CastInfo, specular_n: i32, light: &PointLight) -> (Color, Color) {
    let nudge = info.normal.into_inner() * 0.001;

    // https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-shading/shading-normals
    let distance_to_light = light.ray.origin - info.point_hit;
    let pointing_to_light = Unit::new_normalize(distance_to_light);
    let shadow_ray = &Ray {
      origin: info.point_hit + nudge,
      direction: pointing_to_light,
    };
    let point_to_light_crosses_object = self.cast_to_shadow_casting_shapes(shadow_ray);
    match point_to_light_crosses_object {
      None => {}
      Some(shadow_info) => {
        // only shadow if casted object is nearer than light
        if shadow_info.distance < distance_to_light.norm() {
          return (Color::zero(), Color::zero());
        }
      }
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
        .powi(specular_n);
    (diffuse, specular)
  }

  fn get_reflected_ray(&self, info: &CastInfo) -> Ray {
    let nudge = info.normal.into_inner() * 0.001;

    let reflection = reflect(&info.pointing_to_viewer, &info.normal);
    Ray {
      direction: Unit::new_normalize(reflection),
      origin: info.point_hit + nudge,
    }
  }
  fn get_refracted_ray(&self, info: &CastInfo) -> (f32, f32, Ray) {
    let ray_direction = -info.pointing_to_viewer;
    let ior = {
      match info.material.material_type {
        MaterialType::Refraction { refractive_index } => refractive_index,
        _ => panic!(),
      }
    };
    let nudge = info.normal.into_inner() * 0.001;
    let outside = info.normal.dot(&ray_direction) < 0.;

    let (n_i, n_t, bias, refract_normal) = {
      if outside {
        (1., ior, -nudge, info.normal)
      } else {
        (ior, 1., nudge, -info.normal)
      }
    };

    let (kr, kt) = fresnel(&ray_direction, &refract_normal, n_i, n_t);
    match refract(&ray_direction, &refract_normal, n_i, n_t) {
      Some(refraction) => {
        // println!("{:?}", refraction);
        (
          kr,
          kt,
          Ray {
            direction: Unit::new_normalize(refraction),
            origin: info.point_hit + bias,
          },
        )
      }
      // total internal reflection
      None => {
        println!("{} {}", kr, kt);
        (1., 0., self.get_reflected_ray(&info))
      }
    }
  }
  pub fn cast_to_shapes(&self, ray: &Ray) -> Option<CastInfo> {
    self
      .shapes
      .iter()
      .map(|obj| obj.cast_ray(&ray))
      .fold(None, get_nearest_cast_info)
  }
  pub fn cast_to_shadow_casting_shapes(&self, ray: &Ray) -> Option<CastInfo> {
    self
      .shapes
      .iter()
      .filter(|obj| obj.is_shadow_casting())
      .map(|obj| obj.cast_ray(&ray))
      .fold(None, get_nearest_cast_info)
  }
  pub fn get_color_at_ray(&self, ray: &Ray, depth: i32) -> Color {
    if depth > MAX_RAY_DEPTH {
      return Color::zero();
    }
    match self.cast_to_shapes(ray) {
      None => BACKGROUND,
      Some(info) => {
        use crate::MaterialType::*;
        match info.material.material_type {
          Phong {
            k_specular,
            k_diffuse,
            specular_n,
          } => {
            let (diffuse, specular) = self
              .lights
              .iter()
              .map(|light| self.get_lighting(&info, specular_n, light))
              .fold((Color::zero(), Color::zero()), |(a1, a2), (b1, b2)| {
                (a1 + b1, a2 + b2)
              });
            diffuse * k_diffuse + specular * k_specular
          }
          Reflection { reflectivity } => {
            (self.get_color_at_ray(&self.get_reflected_ray(&info), depth + 1) * reflectivity)
              + (info.material.color * (1. - reflectivity))
          }
          Refraction { .. } => {
            let reflect_color = self.get_color_at_ray(&self.get_reflected_ray(&info), depth + 1);
            let (kr, kt, refracted_ray) = self.get_refracted_ray(&info);
            let refract_color = self.get_color_at_ray(&refracted_ray, depth + 1);
            refract_color * kt + reflect_color * kr
          }
        }
      }
    }
  }

  // pub fn trace(&self, ray: &Ray) -> Option<(CastInfo, Option<Ray>)> {
  //   use crate::MaterialType::*;
  //   match self.cast_to_shapes(ray) {
  //     None => None,
  //     Some(info) => match info.material.material_type {
  //       Diffuse => Some((info, None)),
  //       Reflection => Some((info, Some(self.get_reflected_ray(&info)))),
  //       Refraction { .. } => Some((info, Some(self.get_refracted_ray(&info)))),
  //       Phong { .. } => Some((info, None)),
  //     },
  //   }
  // }
  // pub fn get_color_at_ray(&self, ray: &Ray, depth: i32) -> Color {
  //   let mut trace_result = self.trace(ray);
  //   let mut color = Color::zero();

  //   let mut reflectivity = 1.;
  //   for _ in 0..MAX_RAY_DEPTH {
  //     let mut object_color = Color::zero();
  //     // Multiple lights(https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-shading/shading-multiple-lights)
  //     match trace_result {
  //       None => {
  //         trace_result = None;
  //         color = BACKGROUND;
  //         break;
  //       }
  //       Some((info, next_ray)) => {
  //         use crate::MaterialType::*;
  //         match info.material.material_type {
  //           Diffuse | Phong { .. } => {
  //             let (diffuse, specular) = self
  //               .lights
  //               .iter()
  //               .map(|light| self.get_lighting(&info, light))
  //               .fold((Color::zero(), Color::zero()), |(a1, a2), (b1, b2)| {
  //                 (a1 + b1, a2 + b2)
  //               });
  //             if let Phong {
  //               k_diffuse,
  //               k_specular,
  //               ..
  //             } = info.material.material_type
  //             {
  //               object_color = diffuse * k_diffuse + specular * k_specular;
  //             } else {
  //               object_color = diffuse;
  //             }
  //             trace_result = None;
  //           }
  //           Reflection => {
  //             trace_result = self.trace(&next_ray.unwrap());
  //           }
  //           Refraction { .. } => {
  //             trace_result = self.trace(&next_ray.unwrap());
  //           }
  //         }
  //       }
  //     }

  //     color += object_color * reflectivity;
  //   }
  //   color
  // }
}

// #[cfg(test)]
// mod tests {
//   use super::*;
//   use crate::{material::Material, material::MaterialType, shapes::sphere::Sphere};
//   use nalgebra::{Point3, Vector3};

//   #[test]
//   fn test_refraction() {
//     let sphere = Sphere::new(
//       Point3::new(0., 0., 0.),
//       1.,
//       Material {
//         color: Color::RGB(0, 0, 0),
//         albedo: 1.0,
//         material_type: MaterialType::Refraction {
//           refractive_index: 1.0,
//         },
//       },
//     );
//     let world = World {
//       shapes: vec![&sphere],
//       lights: vec![],
//     };
//     let front = Unit::new_normalize(Vector3::new(0., 0., 1.));
//     let looking_at_sphere = Ray {
//       origin: Point3::new(0., 0., -10.),
//       direction: front,
//     };

//     // assert_eq!(world.get_color_at_ray(looking_at_sphere, 0), BACKGROUND);

//     let (first_cast, second_ray) = world.trace(&looking_at_sphere).unwrap();
//     assert_eq!(first_cast.point_hit, Point3::new(0., 0., -1.));
//     assert_eq!(second_ray.direction, front);

//     let (second_cast, third_ray) = world.trace(&second_ray).unwrap();
//     assert_eq!(second_cast.point_hit, Point3::new(0., 0., 1.));
//     assert_eq!(third_ray.direction, front);
//     // println!("third ray: {:?}", third_ray);
//     // match world.trace(&Ray {
//     //   origin: Point3::new(0., 0., 1.1),
//     //   direction: front,
//     // }) {
//     //   None => {}
//     //   Some(third_cast) => panic!("should not have hitted anything: {:?}", third_cast),
//     // };
//   }
// }
