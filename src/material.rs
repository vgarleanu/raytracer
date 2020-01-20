use crate::hitable::HitRecord;
use crate::ray::Ray;
use crate::texture::Texture;
use crate::vec3::Vec3;
use dyn_clone::DynClone;
use rand::prelude::*;
use std::fmt::Debug as DebugTrait;

pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * v.dot(n) * n
}

pub fn refract(v: Vec3, n: Vec3, ni_over_nt: f64, refracted: &mut Vec3) -> bool {
    let uv = v.unit_vector();
    let dt = uv.dot(n);
    let discriminant = 1.0 - ni_over_nt * ni_over_nt * (1.0 - dt * dt);
    if discriminant > 0.0 {
        refracted.update(ni_over_nt * (uv - n * dt) - n * discriminant.sqrt());
        return true;
    }
    false
}

pub fn schlick(cos: f64, ref_index: f64) -> f64 {
    let mut r0 = (1.0 - ref_index) / (1.0 + ref_index);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cos).powi(5)
}

pub trait Material: Sync + DynClone + Send + DebugTrait {
    fn scatter(
        &self,
        ray_in: &Ray,
        hit_record: &HitRecord,
        attenuation: &mut Vec3,
        scattered: &mut Ray,
    ) -> bool;
    fn emitted(&self, _: f64, _: f64, _: Vec3) -> Vec3 {
        Vec3::with_values(0.0, 0.0, 0.0)
    }
}

dyn_clone::clone_trait_object!(Material);

#[derive(Clone, Debug)]
pub struct Lambertian {
    albedo: Box<dyn Texture>,
}

impl Lambertian {
    pub fn new(albedo: Box<dyn Texture>) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(
        &self,
        ray: &Ray,
        hit_record: &HitRecord,
        attenuation: &mut Vec3,
        scattered: &mut Ray,
    ) -> bool {
        let target = hit_record.p + hit_record.normal + Ray::random_in_sphere();
        scattered.update(Ray::with_values(
            hit_record.p,
            target - hit_record.p,
            None,
            ray.3,
        ));
        attenuation.update(self.albedo.value(hit_record.u, hit_record.v, hit_record.p));
        true
    }
}

#[derive(Clone, Debug)]
pub struct Metal {
    albedo: Box<dyn Texture>,
    fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Box<dyn Texture>, fuzz: f64) -> Self {
        Self {
            albedo,
            fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
        }
    }
}

impl Material for Metal {
    fn scatter(
        &self,
        ray_in: &Ray,
        hit_record: &HitRecord,
        attenuation: &mut Vec3,
        scattered: &mut Ray,
    ) -> bool {
        let reflected = reflect(ray_in.direction().unit_vector(), hit_record.normal);
        scattered.update(Ray::with_values(
            hit_record.p,
            reflected + self.fuzz * Ray::random_in_sphere(),
            Some(ray_in.time()),
            ray_in.3,
        ));
        attenuation.update(self.albedo.value(hit_record.u, hit_record.v, hit_record.p));
        scattered.direction().dot(hit_record.normal) > 0.0
    }
}

#[derive(Clone, Debug)]
pub struct Dielectric {
    ref_index: f64,
}

impl Dielectric {
    pub fn new(ref_index: f64) -> Self {
        Self { ref_index }
    }
}

impl Material for Dielectric {
    fn scatter(
        &self,
        ray_in: &Ray,
        hit_record: &HitRecord,
        attenuation: &mut Vec3,
        scattered: &mut Ray,
    ) -> bool {
        let mut rng = rand::thread_rng();
        let outward_normal;
        let ni_over_nt;
        let reflect_prob;
        let cos;
        let reflected = reflect(ray_in.direction(), hit_record.normal);
        let mut refracted = Vec3::new();
        attenuation.update(Vec3::with_values(1.0, 1.0, 1.0));

        if ray_in.direction().dot(hit_record.normal) > 0.0 {
            outward_normal = -hit_record.normal;
            ni_over_nt = self.ref_index;
            cos = self.ref_index * ray_in.direction().dot(hit_record.normal)
                / ray_in.direction().len();
        } else {
            outward_normal = hit_record.normal;
            ni_over_nt = 1.0 / self.ref_index;
            cos = -ray_in.direction().dot(hit_record.normal) / ray_in.direction().len();
        }

        if refract(
            ray_in.direction(),
            outward_normal,
            ni_over_nt,
            &mut refracted,
        ) {
            reflect_prob = schlick(cos, self.ref_index);
        } else {
            reflect_prob = 1.0;
        }

        if rng.gen::<f64>() < reflect_prob {
            scattered.update(Ray::with_values(hit_record.p, reflected, None, ray_in.3));
        } else {
            scattered.update(Ray::with_values(hit_record.p, refracted, None, ray_in.3));
        }
        true
    }
}

#[derive(Clone, Debug)]
pub struct Blank;

impl Blank {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Blank {
    fn default() -> Self {
        Self::new()
    }
}

impl Material for Blank {
    fn scatter(&self, _: &Ray, _: &HitRecord, _: &mut Vec3, _: &mut Ray) -> bool {
        false
    }
}

#[derive(Clone, Debug)]
pub struct DiffuseLight {
    emit: Box<dyn Texture>,
}

impl DiffuseLight {
    pub fn new(emit: Box<dyn Texture>) -> Self {
        Self { emit }
    }
}

impl Material for DiffuseLight {
    fn scatter(&self, _: &Ray, _: &HitRecord, _: &mut Vec3, _: &mut Ray) -> bool {
        false
    }

    fn emitted(&self, u: f64, v: f64, p: Vec3) -> Vec3 {
        self.emit.value(u, v, p)
    }
}
