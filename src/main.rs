extern crate sdl2;
extern crate nalgebra as na;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use na::{Vector3, Unit};
use na::geometry::{Point2, Point3, Perspective3};

const SCREEN_WIDTH: f32 = 240.0;
const SCREEN_HEIGHT: f32 = 160.0;
const SCALE: f32 = 2.5;

mod ray;
mod sphere;
use ray::Ray;
use sphere::Sphere;


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

        NDCCoords{near: near_ndc_point}//, far: far_ndc_point }
    }
}
impl From<NDCCoords> for ScreenPoint {
    fn from(p: NDCCoords) -> Self {
        Point2::new(p.near.x * SCREEN_WIDTH, p.near.y * SCREEN_HEIGHT)
    }
}


fn main() -> Result<(), String> {
    let mut sphere = Sphere {
        center: Point3::new(0., 0., -10.),
        radius: 1.,
    };

    let mut eye = Point3::new(0., 0., 0.);
    let light = Ray {
        direction: Vector3::new(0., -1., 0.),
        origin: Point3::new(0., 20., 20.),
    };
    let projection   = Perspective3::new(SCREEN_WIDTH / SCREEN_HEIGHT, 3.14 / 2.0, 1.0, 1000.0);


    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("rust-sdl2 demo: Video", (SCALE * SCREEN_WIDTH) as u32, (SCALE * SCREEN_HEIGHT) as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_scale(SCALE, SCALE)?;
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    // canvas.copy(texture, src, dst)
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    sphere.center.x -= 1.
                },
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    sphere.center.x += 1.
                },
                Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                    sphere.center.z -= 1.
                }
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    sphere.center.z += 1.
                }
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    sphere.center.y += 1.
                }
                Event::KeyDown { keycode: Some(Keycode::E), .. } => {
                    sphere.center.y -= 1.
                }
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    eye.x += 1.
                },
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    eye.x -= 1.
                },
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    eye.z -= 1.
                }
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    eye.z += 1.
                }
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    println!("eye: {:?}", eye);
                    println!("sphere: {:?}", sphere.center);
                    // println!(": {:?}", sphere.center);
                }
                _ => {}
            }
        }
    
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        
        for x in 0..(SCREEN_WIDTH as i32) {
            for y in 0..(SCREEN_HEIGHT as i32) {
                let screen_point = Point2::new(x as f32, y as f32);
            
                let ndc: NDCCoords = screen_point.into();
                
                // Unproject them to view-space.
                let near_view_point = projection.unproject_point(&ndc.near);

                let direction = Unit::new_normalize(near_view_point - eye);
                let ray = Ray { origin: eye, direction: direction.into_inner() };
                
                let color = sphere.get_color(&ray, &light);
                canvas.set_draw_color(color);
                canvas.draw_point((x, y))?;
            }
        }
        canvas.present();


        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
        
    }

    Ok(())
}
