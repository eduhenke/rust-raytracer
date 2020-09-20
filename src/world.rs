use crate::ray::Ray;
use crate::shapes::Shape;
use na::Unit;
use sdl2::pixels::Color;

#[derive(Debug)]
pub struct World<'a> {
  pub shapes: Vec<&'a (dyn Shape + Sync)>,
  pub light: Ray,
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
        // https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-shading/shading-normals
        let pointing_to_light = Unit::new_normalize(self.light.origin - info.point_hit);
        let facing_ratio: f32 = info.normal.dot(&pointing_to_light).max(0.);
        let lightness_percentage = (facing_ratio).min(1.0);
        let color = (255. * lightness_percentage) as u8;
        Color::RGB(color, color, color)
      }
    }
  }
}
