extern crate nalgebra as na;
extern crate sdl2;

use crate::shapes::plane::Plane;
use crate::shapes::CastInfo;
use crate::shapes::Shape;
use na::geometry::{Perspective3, Point2, Point3};
use na::{Unit, Vector3};
use rayon::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Instant;

const SCREEN_WIDTH: f32 = 240.0;
const SCREEN_HEIGHT: f32 = 180.0;
const SCALE: f32 = 2.5;

mod ray;
mod shapes;
mod world;
use ray::Ray;
use shapes::{sphere::Sphere, Castable, Movable};
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
    // let far_ndc_point  = Point3::new(p.x / SCREEN_WIDTH - 0.5, -(p.y / SCREEN_HEIGHT - 0.5),  1.0);

    NDCCoords {
      near: near_ndc_point,
    } //, far: far_ndc_point }
  }
}
impl From<NDCCoords> for ScreenPoint {
  fn from(p: NDCCoords) -> Self {
    Point2::new(p.near.x * SCREEN_WIDTH, p.near.y * SCREEN_HEIGHT)
  }
}

fn main() -> Result<(), String> {
  let projection = Perspective3::new(SCREEN_WIDTH / SCREEN_HEIGHT, 3.14 / 2.0, 1.0, 1000.0);

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
    center: Point3::new(0., 0., -10.),
    radius: 1.,
  };
  let plane = Plane {
    normal: Unit::new_normalize(Vector3::new(0., 1., 0.)),
  };
  let shapes: Vec<&(dyn Shape + Sync)> = vec![&sphere, &plane];

  let mut camera = Point3::new(0., 0., 0.);
  let light = Ray {
    direction: Unit::new_normalize(Vector3::new(0., -1., 0.)),
    origin: Point3::new(0., 20., 20.),
  };

  let world = World {
    shapes,
    camera,
    light,
  };

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
        Event::KeyDown {
          keycode: Some(Keycode::Left),
          ..
        } => camera.x += 1.,
        Event::KeyDown {
          keycode: Some(Keycode::Right),
          ..
        } => camera.x -= 1.,
        Event::KeyDown {
          keycode: Some(Keycode::Up),
          ..
        } => camera.z -= 1.,
        Event::KeyDown {
          keycode: Some(Keycode::Down),
          ..
        } => camera.z += 1.,
        Event::KeyDown {
          keycode: Some(Keycode::Space),
          ..
        } => {
          println!("camera: {:?}", camera);
          println!("sphere: {:?}", sphere);
        }
        _ => {}
      }
    }

    let grid: Vec<(i32, i32)> = (0..(SCREEN_WIDTH as i32))
      .flat_map(|x| (0..(SCREEN_HEIGHT as i32)).map(move |y| (x, y)))
      .collect();

    let color_grid: Vec<((i32, i32), Color)> = grid
      .into_par_iter()
      .map(|(x, y)| {
        let screen_point = Point2::new(x as f32, y as f32);

        let ndc: NDCCoords = screen_point.into();

        // Unproject them to view-space.
        let near_view_point = projection.unproject_point(&ndc.near);
        let color = world.get_color_at_ray(&Ray {
          direction: Unit::new_normalize(near_view_point - camera),
          origin: camera,
        });
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
