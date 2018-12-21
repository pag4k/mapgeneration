extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

extern crate rs_perlinnoise;
use rs_perlinnoise::PerlinNoise;

use std::collections::VecDeque;
use std::f32;
use std::fs::File;
use std::io::prelude::*;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let mut map_configuration =
        get_map_config("mapconfig.txt").expect("Unable to get map configuration.");
    let terrains = get_terrain_data("terrains.txt").expect("Unable to get terrain data.");

    let window = video_subsystem
        .window(
            "Height Map Generation",
            map_configuration.width as u32,
            map_configuration.height as u32,
        )
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let noise = PerlinNoise::default();
    let mut map = Map::new(map_configuration.width, map_configuration.height);

    map.generate_noise(&noise, &map_configuration);

    map.find_islands();

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
                    map_configuration.offset += 1;
                    map.generate_noise(&noise, &map_configuration);
                }
                _ => {}
            }
        }

        canvas.clear();
        for y in 0..map_configuration.height {
            for x in 0..map_configuration.width {
                let color = terrains
                    .iter()
                    .find(|terrain| map.z[y][x].0 <= terrain.altitude)
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
    island_count: usize,
    z: Vec<Vec<(f32, usize)>>,
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
            island_count: 0,
            z: vec![vec![(0_f32, 0); width]; height],
        }
    }

    fn generate_noise(&mut self, noise: &PerlinNoise, map_configuration: &MapConfiguration) {
        let scale = if map_configuration.scale > 0_f32 {
            map_configuration.scale
        } else {
            0.0001
        };

        let mut min_altitude = f32::MAX;
        let mut max_altitude = f32::MIN;
        for y in 0..self.height {
            for x in 0..self.width {
                let mut amplitude = 1_f32;
                let mut frequency = 1_f32;
                let mut noise_altitude = 0_f32;
                for i in 0..map_configuration.octaves {
                    let current_offset = map_configuration.offset.pow((i + 1) as u32) as f32;
                    noise_altitude += (noise.perlin(
                        x as f32 * scale * frequency + current_offset,
                        y as f32 * scale * frequency + current_offset,
                    ) * 2_f32
                        - 1_f32)
                        * amplitude;
                    amplitude *= map_configuration.persistance;
                    frequency *= map_configuration.lacunarity;
                }

                if noise_altitude < min_altitude {
                    min_altitude = noise_altitude;
                }
                if noise_altitude > max_altitude {
                    max_altitude = noise_altitude;
                }

                self.z[y][x].0 = noise_altitude;
            }
        }

        //Put back for 0 to 1.
        for y in 0..self.height {
            for x in 0..self.width {
                self.z[y][x].0 = (self.z[y][x].0 - min_altitude) / (max_altitude - min_altitude)
            }
        }
    }

    fn find_islands(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.z[y][x].1 == 0 && self.z[y][x].0 > 0.5 {
                    self.island_count += 1;
                    self.set_island(x, y, self.island_count);
                }
            }
        }

        println!("Number of islands: {}.", self.island_count);
    }

    fn set_island(&mut self, x: usize, y: usize, current_island: usize) {
        let mut queue: VecDeque<(usize, usize)> = VecDeque::default();
        self.z[y][x].1 = current_island;
        queue.push_back((x, y));

        let neighbours: Vec<(isize, isize)> = vec![(1, 0), (0, 1), (-1, 0), (0, -1)];

        while !queue.is_empty() {
            let (x, y) = queue.pop_back().unwrap();
            self.z[y][x].1 = current_island;
            let new_tiles: Vec<(usize, usize)> = neighbours
                .iter()
                .map(|(dx, dy)| (x as isize + *dx, y as isize + *dy))
                .filter(|(x, y)| {
                    0 <= *x && *x < self.width as isize && 0 <= *y && *y < self.height as isize
                })
                .map(|(x, y)| (x as usize, y as usize))
                .filter(|(x, y)| self.z[*y][*x].1 == 0 && self.z[*y][*x].0 > 0.5)
                .collect();
            for (x, y) in new_tiles {
                self.z[y][x].1 = current_island;
                queue.push_back((x, y));
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Terrain {
    name: String,
    altitude: f32,
    color: (u8, u8, u8),
}

fn set_terrain_data(file_name: &str, data: &Vec<Terrain>) -> Result<(), std::io::Error> {
    let mut file = File::create(file_name)?;
    let j = serde_json::to_string(data)?;
    file.write_all(j.as_bytes());
    Ok(())
}

fn get_terrain_data(file_name: &str) -> Result<Vec<Terrain>, std::io::Error> {
    let mut data = String::new();
    let mut file = File::open(file_name)?;
    file.read_to_string(&mut data)?;
    let terrains: Vec<Terrain> = serde_json::from_str(&data)?;
    Ok(terrains)
}

#[derive(Serialize, Deserialize)]
struct MapConfiguration {
    height: usize,
    width: usize,
    offset: usize,
    scale: f32,
    octaves: usize,
    persistance: f32,
    lacunarity: f32,
}

fn set_map_config(file_name: &str, data: &MapConfiguration) -> Result<(), std::io::Error> {
    let mut file = File::create(file_name)?;
    let j = serde_json::to_string(data)?;
    file.write_all(j.as_bytes());
    Ok(())
}

fn get_map_config(file_name: &str) -> Result<MapConfiguration, std::io::Error> {
    let mut data = String::new();
    let mut file = File::open(file_name)?;
    file.read_to_string(&mut data)?;
    let terrains: MapConfiguration = serde_json::from_str(&data)?;
    Ok(terrains)
}
