use crate::aabb::Aabb;
use crate::material::{Lambertian, Material};
use crate::ray::Ray;
use crate::texture::*;
use crate::vec3::Vec3;
use dyn_clone::DynClone;
use rand::Rng;
use std::fmt::Debug as DebugTrait;

pub trait Hitable: DynClone + Send + DebugTrait {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool;
    fn get_material(&self) -> Box<dyn Material>;
    fn bounding_box(&self, t0: f64, t1: f64, bounding_box: &mut Aabb) -> bool;
}

dyn_clone::clone_trait_object!(Hitable);

pub fn surrounding_box(box0: &Aabb, box1: &Aabb) -> Aabb {
    let small = Vec3::with_values(
        box0.min().x().min(box1.min().x()),
        box0.min().y().min(box1.min().y()),
        box0.min().z().min(box1.min().z()),
    );

    let big = Vec3::with_values(
        box0.max().x().min(box1.max().x()),
        box0.max().y().min(box1.max().y()),
        box0.max().z().min(box1.max().z()),
    );
    Aabb::new(small, big)
}

pub struct HitRecord {
    pub u: f64,
    pub v: f64,
    pub t: f64,
    pub p: Vec3,
    pub normal: Vec3,
    pub material: Box<dyn Material>,
}

impl HitRecord {
    pub fn new() -> Self {
        Self {
            u: 0.0,
            v: 0.0,
            t: 0.0,
            p: Vec3::new(),
            normal: Vec3::new(),
            material: Box::new(Lambertian::new(SolidTexture::new(Vec3::new()))),
        }
    }

    pub fn update(&mut self, rhs: &HitRecord) {
        self.t = rhs.t;
        self.p = rhs.p;
        self.u = rhs.u;
        self.v = rhs.v;
        self.normal = rhs.normal;
        self.material = dyn_clone::clone_box(&*rhs.material);
    }
}

#[derive(Clone, Debug)]
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
        Box::new(Lambertian::new(SolidTexture::new(Vec3::new())))
    }

    fn bounding_box(&self, t0: f64, t1: f64, bounding_box: &mut Aabb) -> bool {
        if self.list.len() < 1 {
            return false;
        }

        let mut temp_box = Aabb::new(Vec3::new(), Vec3::new());
        if !self.list[0].bounding_box(t0, t1, &mut temp_box) {
            std::mem::replace(bounding_box, temp_box.clone());
        } else {
            return false;
        };

        for object in self.list.iter() {
            if object.bounding_box(t0, t1, &mut temp_box) {
                std::mem::replace(bounding_box, surrounding_box(bounding_box, &temp_box));
            } else {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Debug)]
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

    fn bounding_box(&self, _: f64, _: f64, bounding_box: &mut Aabb) -> bool {
        std::mem::replace(
            bounding_box,
            Aabb::new(
                self.center - Vec3::with_values(self.radius, self.radius, self.radius),
                self.center + Vec3::with_values(self.radius, self.radius, self.radius),
            ),
        );
        true
    }
}
#[derive(Clone, Debug)]
pub struct MovingSphere {
    radius: f64,
    centers: (Vec3, Vec3),
    pub material: Box<dyn Material>,
    time0: f64,
    time1: f64,
}

impl MovingSphere {
    pub fn with_values(
        centers: (Vec3, Vec3),
        time0: f64,
        time1: f64,
        radius: f64,
        material: Box<dyn Material>,
    ) -> Self {
        Self {
            radius,
            centers,
            time0,
            time1,
            material,
        }
    }

    pub fn center(&self, time: f64) -> Vec3 {
        self.centers.0
            + ((time - self.time0) / (self.time1 - self.time0)) * (self.centers.1 - self.centers.0)
    }
}

impl Hitable for MovingSphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        let oc = r.origin() - self.center(r.time());
        let a = r.direction().dot(r.direction());
        let b = oc.dot(r.direction());
        let c = oc.dot(oc) - self.radius * self.radius;
        let d = b * b - a * c;

        if d > 0.0 {
            let temp = (-b - d.sqrt()) / a;
            if temp < t_max && temp > t_min {
                rec.t = temp;
                rec.p = r.point_at_param(rec.t);
                rec.normal = (rec.p - self.center(r.time())) / self.radius;
                rec.material = self.get_material();
                return true;
            }

            let temp = (-b + d.sqrt()) / a;
            if temp < t_max && temp > t_min {
                rec.t = temp;
                rec.p = r.point_at_param(rec.t);
                rec.normal = (rec.p - self.center(r.time())) / self.radius;
                rec.material = self.get_material();
                return true;
            }
        }
        false
    }

    fn get_material(&self) -> Box<dyn Material> {
        dyn_clone::clone_box(&*self.material)
    }

    fn bounding_box(&self, t0: f64, t1: f64, bounding_box: &mut Aabb) -> bool {
        let box0 = Aabb::new(
            self.center(t0) - Vec3::with_values(self.radius, self.radius, self.radius),
            self.center(t0) + Vec3::with_values(self.radius, self.radius, self.radius),
        );

        let box1 = Aabb::new(
            self.center(t1) - Vec3::with_values(self.radius, self.radius, self.radius),
            self.center(t1) + Vec3::with_values(self.radius, self.radius, self.radius),
        );

        std::mem::replace(bounding_box, surrounding_box(&box0, &box1));
        true
    }
}

#[derive(Clone, Debug)]
pub struct BvhNode {
    left: Box<dyn Hitable>,
    right: Box<dyn Hitable>,
    bounding_box: Aabb,
}

impl BvhNode {
    pub fn new(nodes: &mut Vec<Box<dyn Hitable>>, t0: f64, t1: f64) -> Self {
        let mut rng = rand::thread_rng();
        let axis = (3.0 * rng.gen::<f64>()) as i32;

        if axis == 0 {
            nodes.sort_unstable_by(|a, b| {
                let mut left_box = Aabb::new(Vec3::new(), Vec3::new());
                let mut right_box = Aabb::new(Vec3::new(), Vec3::new());
                let _ = a.bounding_box(0.0, 0.0, &mut left_box)
                    || b.bounding_box(0.0, 0.0, &mut right_box);

                (left_box.min().x() - right_box.min().x())
                    .partial_cmp(&0.0)
                    .unwrap()
            });
        } else if axis == 1 {
            nodes.sort_unstable_by(|a, b| {
                let mut left_box = Aabb::new(Vec3::new(), Vec3::new());
                let mut right_box = Aabb::new(Vec3::new(), Vec3::new());
                let _ = a.bounding_box(0.0, 0.0, &mut left_box)
                    || b.bounding_box(0.0, 0.0, &mut right_box);

                (left_box.min().y() - right_box.min().y())
                    .partial_cmp(&0.0)
                    .unwrap()
            });
        } else {
            nodes.sort_unstable_by(|a, b| {
                let mut left_box = Aabb::new(Vec3::new(), Vec3::new());
                let mut right_box = Aabb::new(Vec3::new(), Vec3::new());
                let _ = a.bounding_box(0.0, 0.0, &mut left_box)
                    || b.bounding_box(0.0, 0.0, &mut right_box);

                (left_box.min().z() - right_box.min().z())
                    .partial_cmp(&0.0)
                    .unwrap()
            });
        }

        if nodes.len() == 1 {
            let mut box_left = Aabb::new(Vec3::new(), Vec3::new());
            nodes[0].bounding_box(t0, t1, &mut box_left);

            Self {
                left: dyn_clone::clone_box(&*nodes[0]),
                right: dyn_clone::clone_box(&*nodes[0]),
                bounding_box: box_left,
            }
        } else if nodes.len() == 2 {
            let mut box_left = Aabb::new(Vec3::new(), Vec3::new());
            let mut box_right = Aabb::new(Vec3::new(), Vec3::new());
            nodes[0].bounding_box(t0, t1, &mut box_left);
            nodes[1].bounding_box(t0, t1, &mut box_right);

            Self {
                left: dyn_clone::clone_box(&*nodes[0]),
                right: dyn_clone::clone_box(&*nodes[1]),
                bounding_box: surrounding_box(&box_left, &box_right),
            }
        } else {
            let mut box_left = Aabb::new(Vec3::new(), Vec3::new());
            let mut box_right = Aabb::new(Vec3::new(), Vec3::new());
            let left = BvhNode::new(&mut nodes.split_off(nodes.len() / 2), t0, t1);
            let right = BvhNode::new(nodes, t0, t1);

            left.bounding_box(t0, t1, &mut box_left);
            right.bounding_box(t0, t1, &mut box_right);

            Self {
                left: Box::new(left),
                right: Box::new(right),
                bounding_box: surrounding_box(&box_left, &box_right),
            }
        }
    }
}

impl Hitable for BvhNode {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        if self.bounding_box.hit(r, t_min, t_max) {
            let mut rec_right = HitRecord::new();
            let mut rec_left = HitRecord::new();

            let hit_left = self.left.hit(r, t_min, t_max, &mut rec_left);
            let hit_right = self.right.hit(r, t_min, t_max, &mut rec_right);

            if hit_left && hit_right {
                if rec_left.t < rec_right.t {
                    std::mem::replace(rec, rec_left);
                } else {
                    std::mem::replace(rec, rec_right);
                }
                return true;
            } else if hit_left {
                std::mem::replace(rec, rec_left);
                return true;
            } else if hit_right {
                std::mem::replace(rec, rec_right);
                return true;
            } else {
                return false;
            }
        }

        false
    }
    fn get_material(&self) -> Box<dyn Material> {
        self.right.get_material()
    }
    fn bounding_box(&self, _: f64, _: f64, bounding_box: &mut Aabb) -> bool {
        std::mem::replace(bounding_box, self.bounding_box.clone());
        true
    }
}
