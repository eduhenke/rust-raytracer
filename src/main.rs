extern crate nalgebra as na;
extern crate sdl2;

use crate::color::Color;
use crate::shapes::plane::Plane;
use crate::shapes::Shape;
use light::PointLight;
use na::{Isometry3, Perspective3, Point2, Point3};
use na::{Unit, Vector3};
use rayon::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::{f32::consts::PI, time::Instant};

const SCREEN_WIDTH: f32 = 200.0;
const SCREEN_HEIGHT: f32 = 160.0;
const SCALE: f32 = 2.5;

mod color;
mod light;
mod ray;
mod shapes;
mod world;
use ray::Ray;
use shapes::sphere::Sphere;
use world::World;

type ScreenPoint = Point2<f32>;
struct NDCCoords {
  near: Point3<f32>,
  // far: Point3<f32>,
}

impl From<ScreenPoint> for NDCCoords {
  fn from(p: ScreenPoint) -> Self {
    // Compute two points in clip-space.
    // "ndc" = normalized device coordinates.
    let near_ndc_point = Point3::new(p.x / SCREEN_WIDTH - 0.5, -(p.y / SCREEN_HEIGHT - 0.5), -1.0);
    // let far_ndc_point = Point3::new(p.x / SCREEN_WIDTH - 0.5, -(p.y / SCREEN_HEIGHT - 0.5), 1.0);

    NDCCoords {
      near: near_ndc_point,
      // far: far_ndc_point,
    }
  }
}
impl From<NDCCoords> for ScreenPoint {
  fn from(p: NDCCoords) -> Self {
    Point2::new(p.near.x * SCREEN_WIDTH, p.near.y * SCREEN_HEIGHT)
  }
}

fn main() -> Result<(), String> {
  let sdl_context = sdl2::init()?;
  let video_subsystem = sdl_context.video()?;

  let window = video_subsystem
    .window(
      "rust-sdl2 demo: Video",
      (SCALE * SCREEN_WIDTH) as u32,
      (SCALE * SCREEN_HEIGHT) as u32,
    )
    .position_centered()
    .opengl()
    .build()
    .map_err(|e| e.to_string())?;

  let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

  canvas.set_scale(SCALE, SCALE)?;
  let mut event_pump = sdl_context.event_pump()?;

  let sphere = Sphere {
    center: Point3::new(0., 0., 0.),
    radius: 1.,
    model: Isometry3::new(Vector3::new(0., 1., -6.), na::zero()),
  };
  let sphere_b = Sphere {
    center: Point3::new(0., 0., 0.),
    radius: 2.,
    model: Isometry3::new(Vector3::new(5., 2., -12.), na::zero()),
  };
  let sphere_c = Sphere {
    center: Point3::new(1.8, 1., -6.5),
    radius: 0.5,
    model: Isometry3::new(na::zero(), na::zero()),
  };
  let plane = Plane {
    normal: Unit::new_normalize(Vector3::new(0., 1., 0.)),
    center: Point3::new(0., 0., 0.),
    // size: (None, None),
    size: (Some(5.0), Some(5.0)),
    model: Isometry3::new(Vector3::new(1., 0., -9.), Vector3::y() * PI / 6.0),
  };
  let shapes: Vec<&(dyn Shape + Sync)> = vec![&sphere, &sphere_b, &sphere_c, &plane];

  let world = World {
    shapes,
    lights: vec![
      PointLight {
        ray: Ray {
          direction: Unit::new_normalize(Vector3::new(0., -1., 0.)),
          origin: Point3::new(-3., 6., -3.),
        },
        color: Color::RGB(200, 140, 0),
        intensity: 500.0,
      },
      PointLight {
        ray: Ray {
          direction: Unit::new_normalize(Vector3::new(0., -1., 0.)),
          origin: Point3::new(2., 10., -12.),
        },
        color: Color::RGB(0, 255, 255),
        intensity: 500.0,
      },
    ],
  };

  // A perspective projection.
  let projection = Perspective3::new(SCREEN_WIDTH / SCREEN_HEIGHT, 3.14 / 2.0, 1.0, 1000.0);

  let eye = Point3::new(0.0, 1.0, 0.0);
  let target = Point3::new(0.0, 0.8, -1.0);
  let view = Isometry3::look_at_rh(&eye, &target, &Vector3::y());

  'running: loop {
    let loop_time = Instant::now();
    // let sphere = shapes.get_mut(0).unwrap();
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. }
        | Event::KeyDown {
          keycode: Some(Keycode::Escape),
          ..
        } => break 'running,
        _ => {}
      }
    }

    // Our camera looks toward the point (1.0, 0.0, 0.0).

    let grid: Vec<(i32, i32)> = (0..(SCREEN_WIDTH as i32))
      .flat_map(|x| (0..(SCREEN_HEIGHT as i32)).map(move |y| (x, y)))
      .collect();
    // let ndc_to_world = projection.inverse();
    // let world_to_camera: Matrix4<f32> = view.inverse().into();
    // let ndc_to_camera = ndc_to_world * world_to_camera;
    let color_grid: Vec<((i32, i32), Color)> = grid
      .into_par_iter()
      .map(|(x, y)| {
        let screen_point = Point2::new(x as f32, y as f32);

        let ndc: NDCCoords = screen_point.into();

        // Unproject them to view-space.
        let world_point = projection.unproject_point(&ndc.near);

        let camera_point = view.inverse_transform_point(&world_point);

        let color = world.get_color_at_ray(
          &Ray {
            direction: Unit::new_normalize(camera_point - eye),
            origin: eye,
          },
          0,
        );
        ((x, y), color)
      })
      .collect();

    for ((x, y), color) in color_grid.into_iter() {
      canvas.set_draw_color(color);
      canvas.draw_point((x as i32, y as i32)).unwrap();
    }

    canvas.present();

    let micros = loop_time.elapsed().as_micros();
    let fps = 1_000_000 / micros;
    println!("elapsed(ms): {} | fps: {}", micros / 1000, fps)
  }

  Ok(())
}
