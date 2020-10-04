use crate::shapes::Shape;
use crate::{color::Color, light::PointLight};
use crate::{ray::Ray, shapes::get_nearest_cast_info};
use na::{Unit, Vector3};
use std::f32::consts::PI;

#[derive(Debug)]
pub struct World<'a> {
  pub shapes: Vec<&'a (dyn Shape + Sync)>,
  pub lights: Vec<PointLight>,
}

const BACKGROUND: Color = Color::RGB(50, 0, 50);
const MAX_REFLECT_DEPTH: i32 = 2;

fn reflect(v: &Vector3<f32>, n: &Unit<Vector3<f32>>) -> Vector3<f32> {
  2.0 * n.dot(v) * n.into_inner() - v
}

impl<'a> World<'a> {
  pub fn get_color_at_ray(&self, ray: &Ray, reflect_depth: i32) -> Color {
    let cast_info = self
      .shapes
      .iter()
      .map(|obj| obj.cast_ray(&ray))
      .fold(None, get_nearest_cast_info);
    match cast_info {
      None => BACKGROUND,
      Some(info) => {
        let kd = 0.8;
        let ks = 1. - kd;
        let mut specular = Color::RGB(0, 0, 0);
        let mut diffuse = Color::RGB(0, 0, 0);
        let nudge = info.normal.into_inner() * 0.01;
        // Multiple lights(https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-shading/shading-multiple-lights)
        for light in &self.lights {
          // https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-shading/shading-normals
          let distance_to_light = light.ray.origin - info.point_hit;
          let pointing_to_light = Unit::new_normalize(distance_to_light);
          let point_to_light_crosses_object = self
            .shapes
            .iter()
            .map(|obj| {
              obj.cast_ray(&Ray {
                origin: info.point_hit + nudge,
                direction: pointing_to_light,
              })
            })
            .fold(None, get_nearest_cast_info);
          match point_to_light_crosses_object {
            Some(shadow_info) => {
              // only shadow if casted object is nearer than light
              if shadow_info.distance < distance_to_light.norm() {
                continue;
              }
            }
            None => {}
          }
          let facing_ratio: f32 = info.normal.dot(&pointing_to_light).max(0.);
          let reflected_light = reflect(&pointing_to_light.into_inner(), &info.normal);

          // intensity at point hit
          let light_intensity_at_point =
            light.intensity / (4. * PI * distance_to_light.norm_squared());

          specular += light.color
            * light_intensity_at_point
            * info
              .pointing_to_viewer
              .into_inner()
              .dot(&reflected_light)
              .max(0.)
              .powi(info.casted.specular_n());
          diffuse += light.color * (info.casted.albedo() * light_intensity_at_point * facing_ratio);
        }
        if reflect_depth < MAX_REFLECT_DEPTH {
          let reflected_direction = reflect(&info.pointing_to_viewer, &info.normal);
          specular += self.get_color_at_ray(
            &Ray {
              direction: Unit::new_normalize(reflected_direction),
              origin: info.point_hit + nudge,
            },
            reflect_depth + 1,
          );
        }
        let color = diffuse * kd + specular * ks;
        color
      }
    }
  }
}
