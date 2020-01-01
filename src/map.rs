use crate::hitable::HitableList;
use crate::hitable::Sphere;
use crate::material::{Dielectric, Lambertian, Material as MaterialClass, Metal};
use crate::vec3::Vec3;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct MapFile {
    pub lookfrom: (f64, f64, f64),
    pub lookat: (f64, f64, f64),
    pub dist_to_focus: f64,
    pub aperture: f64,

    objects: Vec<Object>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Object {
    Sphere {
        position: (f64, f64, f64),
        radius: f64,
        material: Material,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Material {
    Lambertian { albedo: (f64, f64, f64) },
    Dielectric(f64),
    Metal { albedo: (f64, f64, f64), fuzz: f64 },
}

impl MapFile {
    pub fn build_world(&self) -> HitableList {
        let mut world = HitableList::new();

        for object in self.objects.iter().cloned() {
            let vobj = match object {
                Object::Sphere {
                    position,
                    radius,
                    material,
                } => {
                    Sphere::with_values(position.into(), radius, MapFile::build_material(material))
                }
            };

            world.put(Box::new(vobj));
        }
        world
    }

    pub fn build_material(material: Material) -> Box<dyn MaterialClass> {
        match material {
            Material::Lambertian { albedo } => Box::new(Lambertian::new(albedo.into())),
            Material::Dielectric(x) => Box::new(Dielectric::new(x)),
            Material::Metal { albedo, fuzz } => Box::new(Metal::new(albedo.into(), fuzz)),
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
                        objects.push(Object::Sphere {
                            position: center,
                            radius: 0.2,
                            material: Material::Lambertian {
                                albedo: (rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>()),
                            },
                        });
                    } else if pick < 0.95 {
                        objects.push(Object::Sphere {
                            position: center,
                            radius: 0.2,
                            material: Material::Metal {
                                albedo: (rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>()),
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
            material: Material::Metal {
                albedo: (1.0, 1.0, 1.0),
                fuzz: 0.0,
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
                albedo: (0.4, 0.2, 0.1),
            },
        });

        objects.push(Object::Sphere {
            position: (4.0, 1.0, 0.0),
            radius: 1.0,
            material: Material::Metal {
                albedo: (0.7, 0.6, 0.5),
                fuzz: 0.0,
            },
        });

        Self {
            lookfrom: (13.0, 2.0, 3.0),
            lookat: (0.0, 0.0, 0.0),
            dist_to_focus: 10.0,
            aperture: 0.1,
            objects,
        }
    }

    pub fn dump_to_file(&self, file: &str) {
        let mut file = std::fs::File::create(file).unwrap();
        let v = serde_json::to_string(self).unwrap();
        let _ = file.write_all(v.as_bytes());
    }
}
