pub mod camera;
pub mod hitable;
pub mod map;
pub mod material;
pub mod ray;
pub mod vec3;

use camera::Camera;
use image::{ImageBuffer, Pixel, Rgb};
use map::MapFile;
use rand::Rng;
use std::sync::mpsc::channel;
use std::thread::{spawn, JoinHandle};
use vec3::*;

const MUL: u32 = 4;
const RAYS: u32 = 120;
const CORE_CNT: u32 = 12;

fn main() {
    let nx = 200 * MUL;
    let ny = 100 * MUL;
    let ns = RAYS / CORE_CNT;
    let map = MapFile::generate_random();
    let world = map.build_world();
    let mut image = ImageBuffer::new(nx, ny);
    let mut threads: Vec<JoinHandle<()>> = Vec::new();

    let camera = Camera::new(
        map.lookfrom.into(),
        map.lookat.into(),
        Vec3::with_values(0.0, 1.0, 0.0),
        20.0,
        nx as f64 / ny as f64,
        map.aperture,
        map.dist_to_focus,
    );

    let (tx, rx) = channel();

    for _ in 0..CORE_CNT {
        let camera = camera.clone();
        let mut world = world.clone();
        let tx = tx.clone();
        let handle = spawn(move || {
            let mut rng = rand::thread_rng();
            let mut result: Vec<Vec<(u8, u8, u8)>> = {
                let mut x = Vec::new();
                for _ in 0..nx {
                    let mut y = Vec::new();
                    for _ in 0..ny {
                        y.push((0, 0, 0))
                    }
                    x.push(y)
                }
                x
            };
            for j in 0..ny {
                for i in 0..nx {
                    let mut col = Vec3::new();
                    for _ in 0..ns {
                        let u = ((i as f64) + rng.gen::<f64>()) / (nx as f64);
                        let v = (((ny - j) as f64) + rng.gen::<f64>()) / (ny as f64);
                        let ray = camera.get_ray(u, v);
                        col += ray.color(&mut world, 0);
                    }

                    col /= ns as f64;

                    let ir = (255.99 * col.x()) as u8;
                    let ig = (255.99 * col.y()) as u8;
                    let ib = (255.99 * col.z()) as u8;
                    result[i as usize][j as usize] = (ir, ig, ib);
                }
            }
            let _ = tx.send(result);
        });
        threads.push(handle);
    }

    let mut results: Vec<Vec<Vec<(u8, u8, u8)>>> = Vec::new();

    for i in threads.drain(0..) {
        i.join().unwrap();
        results.push(rx.recv().unwrap());
    }

    let blank: Vec<Vec<(u8, u8, u8)>> = {
        let mut x = Vec::new();
        for _ in 0..nx {
            let mut y = Vec::new();
            for _ in 0..ny {
                y.push((0, 0, 0))
            }
            x.push(y)
        }
        x
    };
    let r = results
        .iter()
        .fold(blank, |acc: Vec<Vec<(u8, u8, u8)>>, x| {
            x.iter()
                .zip(acc.iter())
                .map(|(a, b)| {
                    a.iter()
                        .zip(b.iter())
                        .map(|(x, y)| {
                            (
                                x.0 as u16 + y.0 as u16,
                                x.1 as u16 + y.1 as u16,
                                x.2 as u16 + y.2 as u16,
                            )
                        })
                        .map(|x| (x.0 / 2, x.1 / 2, x.2 / 2))
                        .map(|x| (x.0 as u8, x.1 as u8, x.2 as u8))
                        .collect::<Vec<(u8, u8, u8)>>()
                })
                .collect::<Vec<Vec<(u8, u8, u8)>>>()
        });

    for (xi, xp) in r.iter().enumerate() {
        for (yi, yp) in xp.iter().enumerate() {
            image.put_pixel(
                xi as u32,
                yi as u32,
                Rgb::from_channels(yp.0, yp.1, yp.2, 255),
            );
        }
    }
    image.save("image.png").unwrap();
}
