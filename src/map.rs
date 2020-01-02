use crate::hitable::HitableList;
use crate::hitable::{MovingSphere, Sphere};
use crate::material::{Dielectric, Lambertian, Material as MaterialClass, Metal};
use crate::texture::{CheckerTexture, NoiseTexture, SolidTexture, Texture as TextureClass};
use crate::vec3::Vec3;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MapFile {
    pub lookfrom: (f64, f64, f64),
    pub lookat: (f64, f64, f64),
    pub dist_to_focus: f64,
    pub aperture: f64,

    pub objects: Vec<Object>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Object {
    Sphere {
        position: (f64, f64, f64),
        radius: f64,
        material: Material,
    },
    MovingSphere {
        position: (f64, f64, f64),
        shift: (f64, f64, f64),
        radius: f64,
        material: Material,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Material {
    Lambertian { texture: Texture },
    Dielectric(f64),
    Metal { texture: Texture, fuzz: f64 },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Texture {
    SolidTexture(u8, u8, u8),
    CheckerTexture {
        odd: Box<Texture>,
        even: Box<Texture>,
    },
    NoiseTexture {
        scale: f64,
    },
}

impl MapFile {
    pub fn build_world(&self) -> HitableList {
        let mut world = HitableList::new();

        for object in self.objects.iter().cloned() {
            match object {
                Object::Sphere {
                    position,
                    radius,
                    material,
                } => world.put(Box::new(Sphere::with_values(
                    Vec3::from(position),
                    radius,
                    MapFile::build_material(material),
                ))),
                Object::MovingSphere {
                    position,
                    shift,
                    radius,
                    material,
                } => world.put(Box::new(MovingSphere::with_values(
                    (
                        Vec3::from(position),
                        Vec3::from(position) + Vec3::from(shift),
                    ),
                    0.0,
                    1.0,
                    radius,
                    MapFile::build_material(material),
                ))),
            };
        }
        world
    }

    pub fn build_material(material: Material) -> Box<dyn MaterialClass> {
        match material {
            Material::Lambertian { texture } => {
                Box::new(Lambertian::new(MapFile::build_texture(texture)))
            }
            Material::Dielectric(x) => Box::new(Dielectric::new(x)),
            Material::Metal { texture, fuzz } => {
                Box::new(Metal::new(MapFile::build_texture(texture), fuzz))
            }
        }
    }

    pub fn build_texture(texture: Texture) -> Box<dyn TextureClass> {
        match texture {
            Texture::SolidTexture(r, g, b) => {
                SolidTexture::new((r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0).into())
            }
            Texture::CheckerTexture { odd, even } => {
                CheckerTexture::new(MapFile::build_texture(*odd), MapFile::build_texture(*even))
            }
            Texture::NoiseTexture { scale } => NoiseTexture::new(scale),
        }
    }

    pub fn generate_random() -> Self {
        let mut rng = rand::thread_rng();
        let mut objects = Vec::new();

        for a in -11..11 {
            for b in -11..11 {
                let pick = rng.gen::<f64>();
                let center = (
                    a as f64 + 0.9 * rng.gen::<f64>(),
                    0.2f64,
                    b as f64 + 0.9 * rng.gen::<f64>(),
                );
                if (Vec3::from(center) as Vec3 - Vec3::with_values(4.0, 0.2, 0.0)).len() > 0.9 {
                    if pick < 0.8 {
                        objects.push(Object::MovingSphere {
                            position: center,
                            shift: (0.0, 0.5 * rng.gen::<f64>(), 0.0),
                            radius: 0.2,
                            material: Material::Lambertian {
                                texture: Texture::SolidTexture(
                                    rng.gen::<u8>(),
                                    rng.gen::<u8>(),
                                    rng.gen::<u8>(),
                                ),
                            },
                        });
                    } else if pick < 0.95 {
                        objects.push(Object::Sphere {
                            position: center,
                            radius: 0.2,
                            material: Material::Metal {
                                texture: Texture::SolidTexture(
                                    rng.gen::<u8>(),
                                    rng.gen::<u8>(),
                                    rng.gen::<u8>(),
                                ),
                                fuzz: 0.3,
                            },
                        });
                    } else {
                        objects.push(Object::Sphere {
                            position: center,
                            radius: 0.2,
                            material: Material::Dielectric(1.5),
                        });
                    }
                }
            }
        }

        objects.push(Object::Sphere {
            position: (0.0, -1000.0, 0.0),
            radius: 1000.0,
            material: Material::Lambertian {
                texture: Texture::CheckerTexture {
                    odd: Box::new(Texture::SolidTexture(235, 47, 6)),
                    even: Box::new(Texture::SolidTexture(12, 36, 97)),
                },
            },
        });

        objects.push(Object::Sphere {
            position: (0.0, 1.0, 0.0),
            radius: 1.0,
            material: Material::Dielectric(1.5),
        });

        objects.push(Object::Sphere {
            position: (-4.0, 1.0, 0.0),
            radius: 1.0,
            material: Material::Lambertian {
                texture: Texture::SolidTexture(102, 51, 25),
            },
        });

        objects.push(Object::Sphere {
            position: (4.0, 1.0, 0.0),
            radius: 1.0,
            material: Material::Lambertian {
                texture: Texture::NoiseTexture { scale: 0.5 },
            },
        });

        Self {
            lookfrom: (13.0, 2.0, 3.0),
            lookat: (0.0, 0.0, 0.0),
            dist_to_focus: 10.0,
            aperture: 0.0,
            objects,
        }
    }

    pub fn test_map() -> MapFile {
        let mut objects = Vec::new();
        objects.push(Object::Sphere {
            position: (0.0, -1000.0, 0.0),
            radius: 1000.0,
            material: Material::Lambertian {
                texture: Texture::NoiseTexture { scale: 0.5 },
            },
        });

        objects.push(Object::Sphere {
            position: (3.0, 1.0, 0.0),
            radius: 1.0,
            material: Material::Lambertian {
                texture: Texture::NoiseTexture { scale: 1.0 },
            },
        });

        objects.push(Object::Sphere {
            position: (1.0, 1.0, 2.0),
            radius: 1.0,
            material: Material::Lambertian {
                texture: Texture::SolidTexture(255, 255, 255),
            },
        });

        Self {
            lookfrom: (13.0, 2.0, 3.0),
            lookat: (0.0, 0.0, 0.0),
            dist_to_focus: 10.0,
            aperture: 0.0,
            objects,
        }
    }

    pub fn dump_to_file(&self, file: &str) {
        let mut file = std::fs::File::create(file).unwrap();
        let v = serde_json::to_string(self).unwrap();
        let _ = file.write_all(v.as_bytes());
    }
}
