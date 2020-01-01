use crate::material::{Lambertian, Material};
use crate::ray::Ray;
use crate::vec3::Vec3;
use dyn_clone::DynClone;

pub trait Hitable: DynClone + Send {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool;
    fn get_material(&self) -> Box<dyn Material>;
}

dyn_clone::clone_trait_object!(Hitable);

pub struct HitRecord {
    pub t: f64,
    pub p: Vec3,
    pub normal: Vec3,
    pub material: Box<dyn Material>,
}

impl HitRecord {
    pub fn new() -> Self {
        Self {
            t: 0.0,
            p: Vec3::new(),
            normal: Vec3::new(),
            material: Box::new(Lambertian::new(Vec3::new())),
        }
    }

    pub fn update(&mut self, rhs: &HitRecord) {
        self.t = rhs.t;
        self.p = rhs.p;
        self.normal = rhs.normal;
        self.material = dyn_clone::clone_box(&*rhs.material);
    }
}

#[derive(Clone)]
pub struct HitableList {
    list: Vec<Box<dyn Hitable>>,
}

impl HitableList {
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }

    pub fn put(&mut self, object: Box<dyn Hitable>) {
        self.list.push(object);
    }
}

impl Hitable for HitableList {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        let mut hit_anything = false;
        let mut closest_so_far = t_max;
        let mut record = HitRecord::new();

        for i in self.list.iter() {
            if i.hit(r, t_min, closest_so_far, &mut record) {
                hit_anything = true;
                closest_so_far = record.t;
                rec.update(&record);
            }
        }
        hit_anything
    }

    fn get_material(&self) -> Box<dyn Material> {
        Box::new(Lambertian::new(Vec3::new()))
    }
}

#[derive(Clone)]
pub struct Sphere {
    radius: f64,
    center: Vec3,
    pub material: Box<dyn Material>,
}

impl Sphere {
    pub fn with_values(center: Vec3, radius: f64, material: Box<dyn Material>) -> Self {
        Self {
            radius,
            center,
            material,
        }
    }
}

impl Hitable for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        let oc = r.origin() - self.center;
        let a = r.direction().dot(r.direction());
        let b = oc.dot(r.direction());
        let c = oc.dot(oc) - self.radius * self.radius;
        let d = b * b - a * c;

        if d > 0.0 {
            let temp = (-b - d.sqrt()) / a;
            if temp < t_max && temp > t_min {
                rec.t = temp;
                rec.p = r.point_at_param(rec.t);
                rec.normal = (rec.p - self.center) / self.radius;
                rec.material = self.get_material();
                return true;
            }

            let temp = (-b + d.sqrt()) / a;
            if temp < t_max && temp > t_min {
                rec.t = temp;
                rec.p = r.point_at_param(rec.t);
                rec.normal = (rec.p - self.center) / self.radius;
                rec.material = self.get_material();
                return true;
            }
        }
        false
    }

    fn get_material(&self) -> Box<dyn Material> {
        dyn_clone::clone_box(&*self.material)
    }
}
