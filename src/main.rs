pub mod aabb;
pub mod camera;
pub mod hitable;
pub mod map;
pub mod material;
pub mod ray;
pub mod texture;
pub mod vec3;

use camera::Camera;
use clap::clap_app;
use hitable::HitableList;
use image::{imageops::*, ImageBuffer, Pixel, Rgb};
use map::MapFile;
use rand::Rng;
use std::thread::{spawn, JoinHandle};
use vec3::*;

fn main() {
    let matches = clap_app!(raytracer =>
        (version: "0.1")
        (author: "Valerian G. <valerian.garleanu@pm.me>")
        (about: "Small raytracer written purely in rust with support for different objects and materials")
        (@arg MAPFILE: -m --map +takes_value "Specify which map file to load, if no map file is specified, a random one will be generated and dumped to disk")
        (@arg RAYCNT: -r --rays +takes_value default_value("100") "Specify amount of rays to use for this render")
        (@arg XRES: -x +takes_value +required default_value("200") "Specify X resolution of the final render")
        (@arg YRES: -y +takes_value +required default_value("200") "Specify Y resolution of the final render")
        (@arg THREADCNT: --threads default_value("8") "Specify number of threads to use")
        (@arg IMAGEOUT: -o --image-ut +required default_value("image.png") "Specify where to save the rendered image")
        (@arg RENDERDBG: -d "If specified, all lighting enters debug mode")
    ).get_matches();

    let nx = matches
        .value_of("XRES")
        .map(|x| x.parse::<u32>().unwrap_or(200))
        .unwrap_or(200);

    let ny = matches
        .value_of("XRES")
        .map(|x| x.parse::<u32>().unwrap_or(200))
        .unwrap_or(200);

    let thread_count = matches
        .value_of("THREADCNT")
        .map(|x| x.parse::<u32>().unwrap_or(8))
        .unwrap_or(8);

    let total_rays = matches
        .value_of("RAYCNT")
        .map(|x| x.parse::<u32>().unwrap_or(100))
        .unwrap_or(100);

    let debug = matches.is_present("RENDERDBG");

    let outfile = matches.value_of("IMAGEOUT").unwrap();

    let ns = total_rays / thread_count;

    let map = matches
        .value_of("MAPFILE")
        .map_or_else(MapFile::map2, MapFile::load_from_file);

    map.dump_to_file("current_map.json");

    let world = map.build_world();

    let mut image = ImageBuffer::new(nx, ny);
    let mut threads: Vec<JoinHandle<Vec<Vec<(u8, u8, u8)>>>> = Vec::new();

    let camera = Camera::new(
        map.lookfrom.into(),
        map.lookat.into(),
        Vec3::with_values(0.0, 1.0, 0.0),
        40.0,
        nx as f64 / ny as f64,
        map.aperture,
        map.dist_to_focus,
        0.0,
        1.0,
        debug,
    );

    println!("Rendering: {}", map.objects.len());

    for _ in 0..thread_count {
        let camera = camera.clone();
        let world = world.clone();
        let handle = spawn(move || render(camera, world, nx, ny, ns));
        threads.push(handle);
    }

    let results: Vec<Vec<Vec<(u8, u8, u8)>>> =
        threads.drain(0..).map(|x| x.join().unwrap()).collect();

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

    // NOTE: Figure out why the image comes out flipped and rotated requiring manual fix
    let image = rotate90(&image);
    let image = flip_horizontal(&image);
    image.save(outfile).unwrap();
}

fn render(
    camera: Camera,
    mut world: HitableList,
    nx: u32,
    ny: u32,
    ns: u32,
) -> Vec<Vec<(u8, u8, u8)>> {
    let mut rng = rand::thread_rng();
    let mut result = Vec::new();
    for j in 0..ny {
        println!("Render column: {} out of {}", j, ny);
        let mut column = Vec::new();
        for i in 0..nx {
            let mut col = Vec3::new();
            for _ in 0..ns {
                let u = ((i as f64) + rng.gen::<f64>()) / (nx as f64);
                let v = (((ny - j) as f64) + rng.gen::<f64>()) / (ny as f64);
                let ray = camera.get_ray(u, v);
                col += ray.color(&mut world, 0);
            }

            col /= ns as f64;
            col = Vec3::with_values(col.x().min(1.0), col.y().min(1.0), col.z().min(1.0));

            let ir = (255.99 * col.x().sqrt()) as u8;
            let ig = (255.99 * col.y().sqrt()) as u8;
            let ib = (255.99 * col.z().sqrt()) as u8;
            column.push((ir, ig, ib));
        }
        result.push(column);
    }
    result
}
