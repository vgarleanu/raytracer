pub mod camera;
pub mod hitable;
pub mod material;
pub mod ray;
pub mod vec3;

use camera::Camera;
use hitable::{HitableList, Sphere};
use image::{ImageBuffer, Pixel, Rgb};
use material::{Dielectric, Lambertian, Metal};
use rand::Rng;
use std::sync::mpsc::channel;
use std::thread::{spawn, JoinHandle};
use vec3::*;

const MUL: u32 = 8;
const RAYS: u32 = 320;
const CORE_CNT: u32 = 16;

fn main() {
    let nx = 200 * MUL;
    let ny = 100 * MUL;
    let ns = RAYS / CORE_CNT;
    let mut rng = rand::thread_rng();
    let mut image = ImageBuffer::new(nx, ny);
    let mut world = HitableList::new();
    let mut threads: Vec<JoinHandle<()>> = Vec::new();

    for a in -11..11 {
        for b in -11..11 {
            let pick = rng.gen::<f64>();
            let center = Vec3::with_values(
                a as f64 + 0.9 * rng.gen::<f64>(),
                0.2,
                b as f64 + 0.9 * rng.gen::<f64>(),
            );
            if (center - Vec3::with_values(4.0, 0.2, 0.0)).len() > 0.9 {
                if pick < 0.8 {
                    world.put(Box::new(Sphere::with_values(
                        center,
                        0.2,
                        Box::new(Lambertian::new(Vec3::with_values(
                            rng.gen::<f64>(),
                            rng.gen::<f64>(),
                            rng.gen::<f64>(),
                        ))),
                    )));
                } else if pick < 0.95 {
                    world.put(Box::new(Sphere::with_values(
                        center,
                        0.2,
                        Box::new(Metal::new(
                            Vec3::with_values(rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>()),
                            0.3,
                        )),
                    )));
                } else {
                    world.put(Box::new(Sphere::with_values(
                        center,
                        0.2,
                        Box::new(Dielectric::new(1.5)),
                    )));
                }
            }
        }
    }

    world.put(Box::new(Sphere::with_values(
        Vec3::with_values(0.0, -1000.0, 0.0),
        1000.0,
        Box::new(Lambertian::new(Vec3::with_values(0.5, 0.5, 0.5))),
    )));

    world.put(Box::new(Sphere::with_values(
        Vec3::with_values(0.0, 1.0, 0.0),
        1.0,
        Box::new(Dielectric::new(1.5)),
    )));

    world.put(Box::new(Sphere::with_values(
        Vec3::with_values(-4.0, 1.0, 0.0),
        1.0,
        Box::new(Lambertian::new(Vec3::with_values(0.4, 0.2, 0.1))),
    )));

    world.put(Box::new(Sphere::with_values(
        Vec3::with_values(4.0, 1.0, 0.0),
        1.0,
        Box::new(Metal::new(Vec3::with_values(0.7, 0.6, 0.5), 0.0)),
    )));

    let lookfrom = Vec3::with_values(13.0, 2.0, 3.0);
    let lookat = Vec3::with_values(0.0, 0.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;

    let camera = Camera::new(
        lookfrom,
        lookat,
        Vec3::with_values(0.0, 1.0, 0.0),
        20.0,
        nx as f64 / ny as f64,
        aperture,
        dist_to_focus,
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

    println!("{}", threads.len());

    let mut results: Vec<Vec<Vec<(u8, u8, u8)>>> = Vec::new();

    for i in threads.drain(0..) {
        i.join().unwrap();
        results.push(rx.recv().unwrap());
    }

    println!("{} {}", results.len(), threads.len());
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

    println!("{}", r.len());
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
