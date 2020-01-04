use crate::vec3::Vec3;
use dyn_clone::DynClone;
use image::{DynamicImage, GenericImageView, Pixel};
use noise::{NoiseFn, Perlin, Turbulence};
use std::fmt;
use std::fmt::Debug as DebugTrait;

pub trait Texture: Sync + DynClone + Send + DebugTrait {
    fn value(&self, u: f64, v: f64, p: Vec3) -> Vec3;
}

dyn_clone::clone_trait_object!(Texture);

#[derive(Clone, Debug)]
pub struct SolidTexture {
    color: Vec3,
}

impl SolidTexture {
    pub fn new(color: Vec3) -> Box<Self> {
        Box::new(Self { color })
    }
}

impl Texture for SolidTexture {
    fn value(&self, _: f64, _: f64, _: Vec3) -> Vec3 {
        self.color
    }
}

#[derive(Clone, Debug)]
pub struct CheckerTexture {
    odd: Box<dyn Texture>,
    even: Box<dyn Texture>,
}

impl CheckerTexture {
    pub fn new(odd: Box<dyn Texture>, even: Box<dyn Texture>) -> Box<Self> {
        Box::new(Self { odd, even })
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, p: Vec3) -> Vec3 {
        let sines = (10.0 * p.x()).sin() * (10.0 * p.y()).sin() * (10.0 * p.z()).sin();
        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}

#[derive(Clone, Debug)]
pub struct NoiseTexture {
    noise: noise::Turbulence<Perlin>,
    scale: f64,
}

impl NoiseTexture {
    pub fn new(scale: f64) -> Box<Self> {
        Box::new(Self {
            noise: Turbulence::new(Perlin::new())
                .set_frequency(1.5)
                .set_roughness(4),
            scale,
        })
    }

    pub fn turbulence(&self, p: Vec3) -> f64 {
        return self.noise.get([p.x(), p.y(), p.z()]);
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _: f64, _: f64, p: Vec3) -> Vec3 {
        Vec3::with_values(1.0, 1.0, 1.0)
            * 0.5
            * (1.0 + (self.scale * p.z() + 10.0 * self.turbulence(p)).sin())
    }
}

#[derive(Clone)]
pub struct ImageTexture {
    nx: u32,
    ny: u32,
    image: DynamicImage,
}

impl ImageTexture {
    pub fn new(path: &str) -> Box<Self> {
        let image = image::open(path).unwrap();
        let (nx, ny) = image.dimensions();
        Box::new(Self { nx, ny, image })
    }
}

impl fmt::Debug for ImageTexture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ImageBuffer {{ nx: {}, ny: {} }}", self.nx, self.ny)
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _: Vec3) -> Vec3 {
        let mut i = (u * self.nx as f64) as u32;
        let mut j = ((1.0 - v) * self.ny as f64 - 0.001) as u32;
        if i > self.nx - 1 {
            i = self.nx - 1;
        }
        if j > self.ny - 1 {
            j = self.ny - 1
        }
        let (r, g, b, _) = unsafe { self.image.unsafe_get_pixel(i, j).channels4() };

        Vec3::with_values(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0)
    }
}
