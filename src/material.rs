use crate::hitable::HitRecord;
use crate::ray::Ray;
use crate::vec3::Vec3;
use dyn_clone::DynClone;

pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * v.dot(n) * n
}

pub trait Material: DynClone + Send {
    fn scatter(
        &self,
        ray_in: &Ray,
        hit_record: &HitRecord,
        attenuation: &mut Vec3,
        scattered: &mut Ray,
    ) -> bool;
}

dyn_clone::clone_trait_object!(Material);

#[derive(Clone)]
pub struct Lambertian {
    albedo: Vec3,
}

impl Lambertian {
    pub fn new(albedo: Vec3) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(
        &self,
        ray_in: &Ray,
        hit_record: &HitRecord,
        attenuation: &mut Vec3,
        scattered: &mut Ray,
    ) -> bool {
        let target = hit_record.p + hit_record.normal + Ray::random_in_sphere();
        scattered.update(Ray::with_values(hit_record.p, target - hit_record.p));
        attenuation.update(self.albedo);
        true
    }
}

#[derive(Clone)]
pub struct Metal {
    albedo: Vec3,
}

impl Metal {
    pub fn new(albedo: Vec3) -> Self {
        Self { albedo }
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
        scattered.update(Ray::with_values(hit_record.p, reflected));
        attenuation.update(self.albedo);
        scattered.direction().dot(hit_record.normal) > 0.0
    }
}
