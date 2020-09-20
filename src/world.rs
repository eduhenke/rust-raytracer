use crate::ray::Ray;
use crate::shapes::Shape;
use na::Unit;
use sdl2::pixels::Color;

#[derive(Debug)]
pub struct World<'a> {
  pub shapes: Vec<&'a (dyn Shape + Sync)>,
  pub lights: Vec<Ray>,
}

const BACKGROUND: Color = Color::RGB(50, 0, 50);

impl<'a> World<'a> {
  pub fn get_color_at_ray(&self, ray: &Ray) -> Color {
    let cast_info = self
      .shapes
      .iter()
      .map(|obj| {
        let new_origin = obj.inverse_model_matrix().transform_point(&ray.origin);
        let new_direction = obj.inverse_model_matrix().transform_vector(&ray.direction);

        obj.cast_ray(&Ray {
          origin: new_origin,
          direction: Unit::new_normalize(new_direction),
        })
      })
      .fold(None, |acc, val| match acc {
        None => val,
        Some(acc_info) => match val {
          None => Some(acc_info),
          Some(val_info) => {
            if acc_info.distance > val_info.distance {
              return Some(val_info);
            } else {
              return Some(acc_info);
            }
          }
        },
      });
    match cast_info {
      None => BACKGROUND,
      Some(info) => {
        let kd = 0.8;
        let ks = 0.2;
        let specular_n = 20;
        let mut specular = 0.0;
        let mut diffuse = 0.0;
        for light in &self.lights {
          // https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-shading/shading-normals
          let pointing_to_light = Unit::new_normalize(light.origin - info.point_hit);
          let facing_ratio: f32 = info.normal.dot(&pointing_to_light).max(0.);
          let reflected_light =
            ((2.0 * facing_ratio) * info.normal.into_inner()) - pointing_to_light.into_inner();
          specular += info
            .pointing_to_viewer
            .into_inner()
            .dot(&reflected_light)
            .powi(specular_n);
          diffuse += facing_ratio
        }
        let lightness_percentage = (kd * diffuse + ks * specular).min(1.0).max(0.0);
        let color = (255. * lightness_percentage) as u8;
        Color::RGB(color, color, color)
      }
    }
  }
}
