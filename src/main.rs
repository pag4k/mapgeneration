extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

extern crate rs_perlinnoise;
use rs_perlinnoise::PerlinNoise;

use std::f32;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let height: usize = 800;
    let width: usize = 800;

    let regions = vec![
        Terrain {
            height: 0.45,
            color: Color::RGB(0, 0, 255),
        },
        Terrain {
            height: 0.5,
            color: Color::RGB(0, 128, 255),
        },
        Terrain {
            height: 0.55,
            color: Color::RGB(153, 153, 0),
        },
        Terrain {
            height: 0.7,
            color: Color::RGB(51, 204, 0),
        },
        Terrain {
            height: 0.8,
            color: Color::RGB(26, 102, 0),
        },
        Terrain {
            height: 0.9,
            color: Color::RGB(153, 77, 51),
        },
        Terrain {
            height: 0.95,
            color: Color::RGB(153, 51, 51),
        },
        Terrain {
            height: 1.0,
            color: Color::RGB(255, 255, 255),
        },
    ];

    let window = video_subsystem
        .window("Height Map Generation", (width) as u32, (height) as u32)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let noise = PerlinNoise::default();
    let mut map = Map::new(width, height);

    let mut offset = 0;
    let scale: f32 = 0.005_f32;
    let octaves: usize = 4;
    let persistance: f32 = 0.5_f32;
    let lacunarity: f32 = 2_f32;
    map.generate_noise(&noise, offset, scale, octaves, persistance, lacunarity);

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    offset += 1;
                    map.generate_noise(&noise, offset, scale, octaves, persistance, lacunarity);
                }
                _ => {}
            }
        }

        canvas.clear();
        for y in 0..height {
            for x in 0..width {
                let color = regions
                    .iter()
                    .find(|region| map.z[y][x] <= region.height)
                    .unwrap()
                    .color;

                canvas.set_draw_color(color);
                let _ = canvas.fill_rect(Rect::new(x as i32, y as i32, 1, 1));
            }
        }
        canvas.present();
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32));
    }
}

struct Map {
    height: usize,
    width: usize,
    z: Vec<Vec<f32>>,
}

impl Default for Map {
    fn default() -> Self {
        Map::new(256, 256)
    }
}

impl Map {
    fn new(width: usize, height: usize) -> Self {
        Map {
            height: height,
            width: width,
            z: vec![vec![0_f32; width]; height],
        }
    }

    fn generate_noise(
        &mut self,
        noise: &PerlinNoise,
        offset: usize,
        scale: f32,
        octaves: usize,
        persistance: f32,
        lacunarity: f32,
    ) {
        let scale = if scale > 0_f32 { scale } else { 0.0001 };

        let mut min_height = f32::MAX;
        let mut max_height = f32::MIN;
        for y in 0..self.height {
            for x in 0..self.width {
                let mut amplitude = 1_f32;
                let mut frequency = 1_f32;
                let mut noise_height = 0_f32;
                for i in 0..octaves {
                    let current_offset = offset.pow((i + 1) as u32) as f32;
                    noise_height += (noise.perlin(
                        x as f32 * scale * frequency + current_offset,
                        y as f32 * scale * frequency + current_offset,
                    ) * 2_f32
                        - 1_f32)
                        * amplitude;
                    amplitude *= persistance;
                    frequency *= lacunarity;
                }

                if noise_height < min_height {
                    min_height = noise_height;
                }
                if noise_height > max_height {
                    max_height = noise_height;
                }

                self.z[y][x] = noise_height;
            }
        }

        //Put back for 0 to 1.
        for y in 0..self.height {
            for x in 0..self.width {
                self.z[y][x] = (self.z[y][x] - min_height) / (max_height - min_height)
            }
        }
    }
}

struct Terrain {
    height: f32,
    color: Color,
}
