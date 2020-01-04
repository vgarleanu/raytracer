use crate::hitable::HitableList;
use crate::hitable::{
    BoxObject, BvhNode, FlipNormals, Hitable, MovingSphere, RectSliceXy, RectSliceXz, RectSliceYz,
    Sphere,
};
use crate::material::{Dielectric, DiffuseLight, Lambertian, Material as MaterialClass, Metal};
use crate::texture::{
    CheckerTexture, ImageTexture, NoiseTexture, SolidTexture, Texture as TextureClass,
};
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
    RectSliceXy {
        params: (f64, f64, f64, f64, f64),
        material: Material,
    },
    RectSliceXz {
        params: (f64, f64, f64, f64, f64),
        material: Material,
    },
    RectSliceYz {
        params: (f64, f64, f64, f64, f64),
        material: Material,
    },
    FlipNormals(Box<Object>),
    BoxObject {
        p0: (f64, f64, f64),
        p1: (f64, f64, f64),
        material: Material,
    },
    BvhNode {
        objects: Box<Vec<Object>>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Material {
    Lambertian { texture: Texture },
    Dielectric(f64),
    Metal { texture: Texture, fuzz: f64 },
    DiffuseLight { texture: Texture },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Texture {
    SolidTexture(u16, u16, u16),
    CheckerTexture {
        odd: Box<Texture>,
        even: Box<Texture>,
    },
    NoiseTexture {
        scale: f64,
    },
    ImageTexture {
        path: String,
    },
}

impl Object {
    pub fn parse(self) -> Box<dyn Hitable> {
        match self {
            Object::Sphere {
                position,
                radius,
                material,
            } => Box::new(Sphere::with_values(
                Vec3::from(position),
                radius,
                MapFile::build_material(material),
            )),
            Object::MovingSphere {
                position,
                shift,
                radius,
                material,
            } => Box::new(MovingSphere::with_values(
                (
                    Vec3::from(position),
                    Vec3::from(position) + Vec3::from(shift),
                ),
                0.0,
                1.0,
                radius,
                MapFile::build_material(material),
            )),
            Object::RectSliceXy { params, material } => {
                Box::new(RectSliceXy::new(MapFile::build_material(material), params))
            }
            Object::RectSliceXz { params, material } => {
                Box::new(RectSliceXz::new(MapFile::build_material(material), params))
            }
            Object::RectSliceYz { params, material } => {
                Box::new(RectSliceYz::new(MapFile::build_material(material), params))
            }
            Object::FlipNormals(object) => Box::new(FlipNormals::new(object.parse())),
            Object::BoxObject { p0, p1, material } => Box::new(BoxObject::new(
                Vec3::from(p0),
                Vec3::from(p1),
                MapFile::build_material(material),
            )),
            Object::BvhNode { objects } => Box::new(BvhNode::new(
                &mut objects.into_iter().map(Object::parse).collect::<_>(),
                0.0,
                1.0,
            )),
        }
    }
}

impl MapFile {
    pub fn build_world(&self) -> HitableList {
        let mut world = HitableList::new();

        for object in self.objects.iter().cloned() {
            world.put(object.parse())
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
            Material::DiffuseLight { texture } => {
                Box::new(DiffuseLight::new(MapFile::build_texture(texture)))
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
            Texture::ImageTexture { path } => ImageTexture::new(path.as_str()),
        }
    }

    pub fn generate_random() -> Self {
        let mut rng = rand::thread_rng();
        let mut objects = Vec::new();

        for a in -8..8 {
            for b in -8..8 {
                let pick = rng.gen::<f64>();
                let center = (
                    a as f64 + 0.9 * rng.gen::<f64>(),
                    0.2f64,
                    b as f64 + 0.9 * rng.gen::<f64>(),
                );
                if (Vec3::from(center) as Vec3 - Vec3::with_values(4.0, 0.2, 0.0)).len() > 0.9 {
                    if pick < 0.2 {
                        objects.push(Object::Sphere {
                            position: center,
                            radius: 0.2,
                            material: Material::Lambertian {
                                texture: Texture::NoiseTexture { scale: 1.0 },
                            },
                        });
                    } else if pick < 0.4 {
                        objects.push(Object::MovingSphere {
                            position: center,
                            shift: (0.0, 0.0 * rng.gen::<f64>(), 0.0),
                            radius: 0.2,
                            material: Material::Lambertian {
                                texture: Texture::SolidTexture(
                                    rng.gen::<u8>() as u16,
                                    rng.gen::<u8>() as u16,
                                    rng.gen::<u8>() as u16,
                                ),
                            },
                        });
                    } else if pick < 0.7 {
                        objects.push(Object::Sphere {
                            position: center,
                            radius: 0.2,
                            material: Material::Metal {
                                texture: Texture::SolidTexture(
                                    rng.gen::<u8>() as u16,
                                    rng.gen::<u8>() as u16,
                                    rng.gen::<u8>() as u16,
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
            position: (-8.0, 1.0, 0.0),
            radius: 1.0,
            material: Material::Lambertian {
                texture: Texture::NoiseTexture { scale: 1.0 },
            },
        });

        objects.push(Object::Sphere {
            position: (-4.0, 1.0, 0.0),
            radius: 1.0,
            material: Material::Lambertian {
                texture: Texture::SolidTexture(102, 51, 25),
            },
        });

        objects.push(Object::Sphere {
            position: (0.0, 1.0, 0.0),
            radius: 1.0,
            material: Material::Dielectric(1.5),
        });

        /*
        objects.push(Object::Sphere {
            position: (4.0, 1.0, 0.0),
            radius: 1.0,
            material: Material::Metal {
                texture: Texture::SolidTexture(255, 255, 255),
                fuzz: 0.0,
            },
        });*/

        objects.push(Object::Sphere {
            position: (4.0, 1.0, 0.0),
            radius: 1.0,
            material: Material::Lambertian {
                texture: Texture::ImageTexture {
                    path: "./textures/earthmap.jpg".into(),
                },
            },
        });

        Self {
            lookfrom: (14.0, 4.0, 7.0),
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
                texture: Texture::NoiseTexture { scale: 1.0 },
            },
        });

        objects.push(Object::Sphere {
            position: (0.0, -2.0, 0.0),
            radius: 2.0,
            material: Material::Lambertian {
                texture: Texture::NoiseTexture { scale: 1.0 },
            },
        });

        objects.push(Object::Sphere {
            position: (0.0, 7.0, 0.0),
            radius: 2.0,
            material: Material::DiffuseLight {
                texture: Texture::SolidTexture(1020, 1020, 1020),
            },
        });

        objects.push(Object::RectSliceXy {
            params: (0.0, 3.0, 0.0, 3.0, -2.0),
            material: Material::DiffuseLight {
                texture: Texture::SolidTexture(1020, 1020, 1020),
            },
        });

        Self {
            lookfrom: (40.0, 7.0, 0.0),
            lookat: (0.0, 0.0, 0.0),
            dist_to_focus: 10.0,
            aperture: 0.0,
            objects,
        }
    }

    pub fn cornell_box() -> MapFile {
        let mut objects = Vec::new();
        let red_material = Material::Lambertian {
            texture: Texture::SolidTexture(255, 0, 0),
        };
        let white_material = Material::Lambertian {
            texture: Texture::SolidTexture(255, 255, 255),
        };

        let green_material = Material::Lambertian {
            texture: Texture::SolidTexture(0, 255, 0),
        };

        let light_material = Material::DiffuseLight {
            texture: Texture::SolidTexture(255 * 15, 255 * 15, 255 * 15),
        };

        let light_metal = Material::Metal {
            texture: Texture::SolidTexture(255, 255, 255),
            fuzz: 0.0,
        };

        objects.push(Object::FlipNormals(Box::new(Object::RectSliceYz {
            params: (0.0, 555.0, 0.0, 555.0, 555.0),
            material: green_material,
        })));

        objects.push(Object::RectSliceYz {
            params: (0.0, 555.0, 0.0, 555.0, 0.0),
            material: red_material,
        });

        objects.push(Object::RectSliceXz {
            params: (213.0, 343.0, 227.0, 332.0, 554.0),
            material: light_material,
        });

        objects.push(Object::FlipNormals(Box::new(Object::RectSliceXz {
            params: (0.0, 555.0, 0.0, 555.0, 555.0),
            material: white_material.clone(),
        })));

        objects.push(Object::RectSliceXz {
            params: (0.0, 555.0, 0.0, 555.0, 0.0),
            material: white_material.clone(),
        });

        objects.push(Object::FlipNormals(Box::new(Object::RectSliceXy {
            params: (0.0, 555.0, 0.0, 555.0, 555.0),
            material: white_material.clone(),
        })));

        objects.push(Object::BoxObject {
            p0: (130.0, 0.0, 65.0),
            p1: (295.0, 165.0, 230.0),
            material: white_material.clone(),
        });

        objects.push(Object::BoxObject {
            p0: (256.0, 0.0, 295.0),
            p1: (430.0, 330.0, 460.0),
            material: white_material.clone(),
        });

        Self {
            lookfrom: (278.0, 278.0, -800.0),
            lookat: (278.0, 278.0, 0.0),
            dist_to_focus: 10.0,
            aperture: 0.0,
            objects,
        }
    }

    pub fn map2() -> MapFile {
        let mut ground_box = Vec::new();
        let mut objects = Vec::new();
        let white = Material::Lambertian {
            texture: Texture::SolidTexture(186, 186, 186),
        };
        let ground = Material::Lambertian {
            texture: Texture::SolidTexture(122, 212, 135),
        };

        let mut rng = rand::thread_rng();

        for i in 0..20 {
            for j in 0..20 {
                let w = 100.0;
                let x0 = 500.0 - (i as f64) * w;
                let z0 = 1500.0 - (j as f64) * w;
                let y0 = 0.0;
                let x1 = x0 + w;
                let y1 = 100.0 * (rng.gen::<f64>() + 0.01);
                let z1 = z0 + w;

                ground_box.push(Object::BoxObject {
                    p0: (x0, y0, z0),
                    p1: (x1, y1, z1),
                    material: ground.clone(),
                });
            }
        }

        ground_box.push(Object::BoxObject {
            p0: (165.0, 0.0, -20.0),
            p1: (300.0, 105.0, 150.0),
            material: ground.clone(),
        });

        objects.push(Object::BvhNode {
            objects: Box::new(ground_box),
        });

        let l = 255 * 7;

        objects.push(Object::RectSliceXz {
            params: (123.0, 423.0, 147.0, 412.0, 554.0),
            material: Material::DiffuseLight {
                texture: Texture::SolidTexture(l, l, l),
            },
        });

        objects.push(Object::MovingSphere {
            position: (400.0, 400.0, 400.0),
            shift: (30.0, 0.0, 0.0),
            radius: 50.0,
            material: Material::Lambertian {
                texture: Texture::SolidTexture(178, 76, 25),
            },
        });

        objects.push(Object::Sphere {
            position: (250.0, 150.0, 45.0),
            radius: 50.0,
            material: Material::Dielectric(1.5),
        });

        objects.push(Object::Sphere {
            position: (0.0, 150.0, 145.0),
            radius: 50.0,
            material: Material::Metal {
                texture: Texture::SolidTexture(204, 204, 230),
                fuzz: 10.0,
            },
        });

        objects.push(Object::Sphere {
            position: (400.0, 210.0, 400.0),
            radius: 100.0,
            material: Material::Lambertian {
                texture: Texture::ImageTexture {
                    path: "./textures/earthmap.jpg".into(),
                },
            },
        });

        objects.push(Object::Sphere {
            position: (220.0, 280.0, 300.0),
            radius: 80.0,
            material: Material::Lambertian {
                texture: Texture::NoiseTexture { scale: 1.0 },
            },
        });

        Self {
            lookat: (278.0, 278.0, 0.0),
            lookfrom: (478.0, 278.0, -600.0),
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
