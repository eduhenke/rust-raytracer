extern crate nalgebra as na;
extern crate sdl2;

use crate::color::Color;
use crate::shapes::plane::Plane;
use crate::shapes::Shape;
use core::f32::consts::FRAC_PI_2;
use core::f32::consts::PI;
use light::PointLight;
use material::{Material, MaterialType};
use na::geometry::Rotation3;
use na::{Isometry3, Perspective3, Point2, Point3};
use na::{Unit, Vector3};
use nalgebra::{Quaternion, UnitQuaternion};
use rayon::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;

const SCREEN_WIDTH: f32 = 400.0;
const SCREEN_HEIGHT: f32 = 300.0;
const SCALE: f32 = 2.;

mod color;
mod light;
mod material;
mod ops;
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

#[derive(Copy, Clone)]
struct Scene<'a> {
  pub projection: Perspective3<f32>,
  pub world: &'a World<'a>,
  pub eye: Point3<f32>,
  pub target: Point3<f32>,
  pub up: Vector3<f32>,
  pub theta_x: f32,
  pub theta_y: f32,
}

fn render(
  (x, y): (i32, i32),
  Scene {
    projection,
    world,
    eye,
    target,
    up,
    theta_x,
    theta_y,
  }: &Scene,
) -> color::Color {
  let view = &UnitQuaternion::from_euler_angles(*theta_x, *theta_y, 0.).inverse()
    * Isometry3::look_at_rh(&eye, &target, &up);
  let screen_point = Point2::new(x as f32, y as f32);

  let ndc: NDCCoords = screen_point.into();
  // Unproject them to view-space.
  let world_point = projection.unproject_point(&ndc.near);

  let camera_point = view.inverse_transform_point(&world_point);

  world.get_color_at_ray(
    &Ray {
      direction: Unit::new_normalize(camera_point - eye),
      origin: *eye,
    },
    0,
  )
}

fn move_camera(scene: Scene, translation: Vector3<f32>) -> Scene {
  let mut next_scene = scene;
  let rotated_translation = &UnitQuaternion::from_euler_angles(scene.theta_x, scene.theta_y, 0.)
    .transform_vector(&translation);
  next_scene.eye += rotated_translation;
  next_scene.target += rotated_translation;
  next_scene
}

fn rotate_camera(scene: Scene, direction: Vector3<f32>, radians: f32) -> Scene {
  let mut next_scene = scene;

  // next_scene.theta_x = (next_scene.theta_x.tan() + (direction.x * radians).tan()).atan();
  next_scene.theta_x = (next_scene.theta_x + direction.x * radians)
    .max(-FRAC_PI_2)
    .min(FRAC_PI_2);
  next_scene.theta_y += direction.y * radians;
  next_scene
}

const MOVE_DELTA: f32 = 0.5;
const ROTATION_DELTA: f32 = 10.0f32 * (PI / 180.0f32);

fn handle_input<'a>(
  scene: Scene<'a>,
  mouse_clicked: &mut bool,
  event: sdl2::event::Event,
) -> Option<Scene<'a>> {
  match event {
    Event::Quit { .. } => return None,
    Event::KeyDown {
      keycode: Some(key), ..
    } => {
      use Keycode::*;
      match key {
        Escape => return None,
        W => return Some(move_camera(scene, Vector3::new(0., 0., -MOVE_DELTA))),
        S => return Some(move_camera(scene, Vector3::new(0., 0., MOVE_DELTA))),
        A => return Some(move_camera(scene, Vector3::new(-MOVE_DELTA, 0., 0.))),
        D => return Some(move_camera(scene, Vector3::new(MOVE_DELTA, 0., 0.))),

        Q => return Some(move_camera(scene, Vector3::new(0., MOVE_DELTA, 0.))),
        E => return Some(move_camera(scene, Vector3::new(0., -MOVE_DELTA, 0.))),

        Z => return Some(rotate_camera(scene, Vector3::y(), ROTATION_DELTA)),
        X => return Some(rotate_camera(scene, Vector3::y(), -ROTATION_DELTA)),
        C => return Some(rotate_camera(scene, Vector3::x(), ROTATION_DELTA)),
        V => return Some(rotate_camera(scene, Vector3::x(), -ROTATION_DELTA)),
        _ => {}
      };
    }
    Event::MouseWheel { y, .. } => {
      return Some(move_camera(scene, Vector3::new(0., 0., -y as f32)))
    }
    Event::MouseButtonDown { .. } => *mouse_clicked = true,
    Event::MouseButtonUp { .. } => *mouse_clicked = false,
    Event::MouseMotion { xrel, yrel, .. } if *mouse_clicked => {
      let y_rotation = Vector3::y() * (-xrel as f32) * (PI / (SCREEN_WIDTH * SCALE));
      let x_rotation = Vector3::x() * (-yrel as f32) * (PI / (SCREEN_HEIGHT * SCALE));
      let axis = Unit::new_normalize(x_rotation + y_rotation);
      return Some(rotate_camera(
        scene,
        axis.into_inner(),
        (x_rotation + y_rotation).norm(),
      ));
    }
    _ => {}
  }
  Some(scene)
}

fn get_next_scene<'a>(
  last_scene: Option<Scene<'a>>,
  world: &'a world::World,
  event_pump: &mut sdl2::EventPump,
  mut mouse_clicked: &mut bool,
) -> Option<Scene<'a>> {
  match last_scene {
    None => {
      let eye = Point3::new(0.0, 1.0, 0.0);
      let target = Point3::new(0.0, 1.0, -1.0);
      Some(Scene {
        // A perspective projection.
        projection: Perspective3::new(SCREEN_WIDTH / SCREEN_HEIGHT, 3.14 / 2.0, 1.0, 1000.0),
        // view: Isometry3::look_at_rh(&eye, &target, &Vector3::y()),
        eye,
        target,
        up: Vector3::y(),
        world,
        theta_x: 0.,
        theta_y: 0.,
      })
    }
    Some(scene) => event_pump
      .poll_iter()
      .fold(Some(scene), |last_scene, event| match last_scene {
        None => None,
        Some(scene) => handle_input(scene, &mut mouse_clicked, event),
      }),
  }
}

fn main() -> Result<(), String> {
  let sdl_context = sdl2::init()?;
  let video_subsystem = sdl_context.video()?;

  let window = video_subsystem
    .window(
      "rust-raytracer",
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

  let shiny_material = Material {
    albedo: 1.0,
    color: Color::RGB(0, 0, 0), // TODO
    material_type: MaterialType::Phong {
      specular_n: 30,
      k_diffuse: 0.7,
      k_specular: 0.3,
    },
  };
  // let mirror_material = Material {
  //   albedo: 1.0,
  //   color: Color::RGB(0, 0, 0), // TODO
  //   material_type: MaterialType::Reflection { reflectivity: 1.0 },
  // };
  let transparent_material = Material {
    albedo: 1.0,
    color: Color::RGB(0, 0, 0), // TODO
    material_type: MaterialType::Refraction {
      refractive_index: 1.03,
    },
  };
  let opaque_material = Material {
    albedo: 1.0,
    color: Color::RGB(0, 0, 0), // TODO
    material_type: MaterialType::Phong {
      specular_n: 1,
      k_diffuse: 1.0,
      k_specular: 0.0,
    },
  };

  let sphere = Sphere::new(Point3::new(0.6, 1., -6.), 1., transparent_material);
  let sphere_b = Sphere::new(Point3::new(3., 2.5, -12.), 2., shiny_material);
  let sphere_c = Sphere::new(Point3::new(-1., 1., -6.5), 0.5, opaque_material);
  let floor = Plane::new(
    Unit::new_normalize(Vector3::new(0., 1., 0.)),
    Point3::new(0., 0., -10.),
    (Some(12.), Some(10.)),
    na::zero(),
    shiny_material,
  );
  let shapes: Vec<&(dyn Shape + Sync)> = vec![&sphere, &sphere_b, &sphere_c, &floor];

  let world = World {
    shapes,
    lights: vec![
      PointLight {
        ray: Ray {
          direction: Unit::new_normalize(Vector3::new(0., -1., 0.)),
          origin: Point3::new(-6., 10., 3.),
        },
        color: Color::RGB(200, 140, 0),
        intensity: 1000.0,
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

  let mut scene: Option<Scene> = None;
  let mut mouse_clicked = false;
  'running: loop {
    let loop_time = Instant::now();
    match get_next_scene(scene, &world, &mut event_pump, &mut mouse_clicked) {
      None => break 'running,
      Some(next_scene) => scene = Some(next_scene),
    };

    let grid: Vec<(i32, i32)> = (0..(SCREEN_WIDTH as i32))
      .flat_map(|x| (0..(SCREEN_HEIGHT as i32)).map(move |y| (x, y)))
      .collect();

    let color_grid: Vec<((i32, i32), Color)> = grid
      .into_par_iter()
      // .into_iter()
      .map(|(x, y)| ((x, y), render((x, y), &scene.unwrap())))
      .collect();

    for ((x, y), color) in color_grid.into_iter() {
      canvas.set_draw_color(color);
      canvas.draw_point((x as i32, y as i32)).unwrap();
    }

    canvas.present();

    let micros = loop_time.elapsed().as_micros();
    let fps = 1_000_000 / micros;

    println!("elapsed(ms): {} | fps: {}", micros / 1000, fps,);
  }

  Ok(())
}
