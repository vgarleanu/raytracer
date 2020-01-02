use crate::vec3::Vec3;
use dyn_clone::DynClone;
use noise::{NoiseFn, Perlin, Turbulence};
use std::fmt::Debug as DebugTrait;

pub trait Texture: DynClone + Send + DebugTrait {
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
        /*
        Box::new(Self {
            noise: Perlin::new(),
            scale,
        })
        */
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
