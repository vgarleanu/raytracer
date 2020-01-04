use crate::aabb::Aabb;
use crate::material::{Blank, Lambertian, Material};
use crate::ray::Ray;
use crate::texture::*;
use crate::vec3::Vec3;
use dyn_clone::DynClone;
use rand::Rng;
use std::fmt::Debug as DebugTrait;

pub trait Hitable: DynClone + Send + DebugTrait {
    fn hit(
        &self,
        r: &Ray,
        t_min: f64,
        t_max: f64,
        rec: &mut HitRecord,
    ) -> (bool, &Box<dyn Material>);
    fn bounding_box(&self, t0: f64, t1: f64, bounding_box: &mut Aabb) -> bool;
    fn get_material(&self) -> &Box<dyn Material>;
}

dyn_clone::clone_trait_object!(Hitable);

pub fn surrounding_box(box0: &Aabb, box1: &Aabb) -> Aabb {
    let small = Vec3::with_values(
        box0.min().x().min(box1.min().x()),
        box0.min().y().min(box1.min().y()),
        box0.min().z().min(box1.min().z()),
    );

    let big = Vec3::with_values(
        box0.max().x().max(box1.max().x()),
        box0.max().y().max(box1.max().y()),
        box0.max().z().max(box1.max().z()),
    );
    Aabb::new(small, big)
}

#[derive(Debug)]
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
    _m: Box<dyn Material>,
}

impl HitableList {
    pub fn new() -> Self {
        Self {
            list: Vec::new(),
            _m: Box::new(Blank::new()),
        }
    }

    pub fn put(&mut self, object: Box<dyn Hitable>) {
        self.list.push(object);
    }
}

impl Hitable for HitableList {
    fn hit(
        &self,
        r: &Ray,
        t_min: f64,
        t_max: f64,
        rec: &mut HitRecord,
    ) -> (bool, &Box<dyn Material>) {
        let mut hit_anything = false;
        let mut closest_so_far = t_max;
        let mut record = HitRecord::new();
        let mut material_ptr = self.list[0].get_material();

        for i in self.list.iter() {
            if i.hit(r, t_min, closest_so_far, &mut record).0 {
                hit_anything = true;
                closest_so_far = record.t;
                rec.update(&record);
                material_ptr = i.get_material();
            }
        }
        (hit_anything, material_ptr)
    }

    fn get_material(&self) -> &Box<dyn Material> {
        // NOTE: only ever used by BoxObject
        self.list[0].get_material()
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

    pub fn get_sphere_uv(p: Vec3) -> (f64, f64) {
        let phi = p.z().atan2(p.x());
        let theta = p.y().asin();

        let u = 1.0 - (phi + std::f64::consts::PI) / (2.0 * std::f64::consts::PI);
        let v = (theta + std::f64::consts::PI / 2.0) / std::f64::consts::PI;

        (u, v)
    }
}

impl Hitable for Sphere {
    fn hit(
        &self,
        r: &Ray,
        t_min: f64,
        t_max: f64,
        rec: &mut HitRecord,
    ) -> (bool, &Box<dyn Material>) {
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
                let (u, v) = Self::get_sphere_uv((rec.p - self.center) / self.radius);
                rec.u = u;
                rec.v = v;
                rec.normal = (rec.p - self.center) / self.radius;
                return (true, &self.material);
            }

            let temp = (-b + d.sqrt()) / a;
            if temp < t_max && temp > t_min {
                rec.t = temp;
                rec.p = r.point_at_param(rec.t);
                let (u, v) = Self::get_sphere_uv((rec.p - self.center) / self.radius);
                rec.u = u;
                rec.v = v;
                rec.normal = (rec.p - self.center) / self.radius;
                return (true, &self.material);
            }
        }
        (false, &self.material)
    }

    fn get_material(&self) -> &Box<dyn Material> {
        &self.material
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
    fn hit(
        &self,
        r: &Ray,
        t_min: f64,
        t_max: f64,
        rec: &mut HitRecord,
    ) -> (bool, &Box<dyn Material>) {
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
                return (true, &self.material);
            }

            let temp = (-b + d.sqrt()) / a;
            if temp < t_max && temp > t_min {
                rec.t = temp;
                rec.p = r.point_at_param(rec.t);
                rec.normal = (rec.p - self.center(r.time())) / self.radius;
                return (true, &self.material);
            }
        }
        (false, &self.material)
    }

    fn get_material(&self) -> &Box<dyn Material> {
        &self.material
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
            nodes.sort_unstable_by(|b, a| {
                let mut left_box = Aabb::new(Vec3::new(), Vec3::new());
                let mut right_box = Aabb::new(Vec3::new(), Vec3::new());
                let _ = a.bounding_box(0.0, 0.0, &mut left_box)
                    || b.bounding_box(0.0, 0.0, &mut right_box);

                (left_box.min().x() - right_box.min().x())
                    .partial_cmp(&0.0)
                    .unwrap()
            });
        } else if axis == 1 {
            nodes.sort_unstable_by(|b, a| {
                let mut left_box = Aabb::new(Vec3::new(), Vec3::new());
                let mut right_box = Aabb::new(Vec3::new(), Vec3::new());
                let _ = a.bounding_box(0.0, 0.0, &mut left_box)
                    || b.bounding_box(0.0, 0.0, &mut right_box);

                (left_box.min().y() - right_box.min().y())
                    .partial_cmp(&0.0)
                    .unwrap()
            });
        } else {
            nodes.sort_unstable_by(|b, a| {
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
    fn hit(
        &self,
        r: &Ray,
        t_min: f64,
        t_max: f64,
        rec: &mut HitRecord,
    ) -> (bool, &Box<dyn Material>) {
        if self.bounding_box.hit(r, t_min, t_max) {
            let mut rec_right = HitRecord::new();
            let mut rec_left = HitRecord::new();

            let hit_left = self.left.hit(r, t_min, t_max, &mut rec_left).0;
            let hit_right = self.right.hit(r, t_min, t_max, &mut rec_right).0;

            if hit_left && hit_right {
                if rec_left.t < rec_right.t {
                    std::mem::replace(rec, rec_left);
                    return (true, self.left.get_material());
                } else {
                    std::mem::replace(rec, rec_right);
                    return (true, self.right.get_material());
                }
            } else if hit_left {
                std::mem::replace(rec, rec_left);
                return (true, self.left.get_material());
            } else if hit_right {
                std::mem::replace(rec, rec_right);
                return (true, self.right.get_material());
            } else {
                return (false, self.left.get_material());
            }
        }

        (false, self.left.get_material())
    }

    fn get_material(&self) -> &Box<dyn Material> {
        self.left.get_material()
    }

    fn bounding_box(&self, _: f64, _: f64, bounding_box: &mut Aabb) -> bool {
        std::mem::replace(bounding_box, self.bounding_box.clone());
        true
    }
}

#[derive(Clone, Debug)]
pub struct RectSliceXy {
    material: Box<dyn Material>,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k: f64,
}

impl RectSliceXy {
    pub fn new(material: Box<dyn Material>, params: (f64, f64, f64, f64, f64)) -> Self {
        let (x0, x1, y0, y1, k) = params;

        Self {
            material,
            x0,
            x1,
            y0,
            y1,
            k,
        }
    }
}

impl Hitable for RectSliceXy {
    fn hit(
        &self,
        r: &Ray,
        t_min: f64,
        t_max: f64,
        rec: &mut HitRecord,
    ) -> (bool, &Box<dyn Material>) {
        let t = (self.k - r.origin().z()) / r.direction().z();
        if t < t_min || t > t_max {
            return (false, self.get_material());
        }

        let x = r.origin().x() + t * r.direction().x();
        let y = r.origin().y() + t * r.direction().y();

        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return (false, self.get_material());
        }

        rec.u = (x - self.x0) / (self.x1 - self.x0);
        rec.v = (y - self.y0) / (self.y1 - self.y0);
        rec.t = t;
        rec.p = r.point_at_param(t);
        rec.normal = Vec3::with_values(0.0, 0.0, 1.0);

        (true, self.get_material())
    }

    fn get_material(&self) -> &Box<dyn Material> {
        &self.material
    }

    fn bounding_box(&self, _: f64, _: f64, bounding_box: &mut Aabb) -> bool {
        std::mem::replace(
            bounding_box,
            Aabb::new(
                Vec3::with_values(self.x0, self.y0, self.k - 0.0001),
                Vec3::with_values(self.x1, self.y1, self.k + 0.0001),
            ),
        );
        true
    }
}

#[derive(Clone, Debug)]
pub struct RectSliceXz {
    material: Box<dyn Material>,
    x0: f64,
    x1: f64,
    z0: f64,
    z1: f64,
    k: f64,
}

impl RectSliceXz {
    pub fn new(material: Box<dyn Material>, params: (f64, f64, f64, f64, f64)) -> Self {
        let (x0, x1, z0, z1, k) = params;

        Self {
            material,
            x0,
            x1,
            z0,
            z1,
            k,
        }
    }
}

impl Hitable for RectSliceXz {
    fn hit(
        &self,
        r: &Ray,
        t_min: f64,
        t_max: f64,
        rec: &mut HitRecord,
    ) -> (bool, &Box<dyn Material>) {
        let t = (self.k - r.origin().y()) / r.direction().y();
        if t < t_min || t > t_max {
            return (false, self.get_material());
        }

        let x = r.origin().x() + t * r.direction().x();
        let z = r.origin().z() + t * r.direction().z();

        if x < self.x0 || x > self.x1 || z < self.z0 || z > self.z1 {
            return (false, self.get_material());
        }

        rec.u = (x - self.x0) / (self.x1 - self.x0);
        rec.v = (z - self.z0) / (self.z1 - self.z0);
        rec.t = t;
        rec.p = r.point_at_param(t);
        rec.normal = Vec3::with_values(0.0, 1.0, 0.0);

        (true, self.get_material())
    }

    fn get_material(&self) -> &Box<dyn Material> {
        &self.material
    }

    fn bounding_box(&self, _: f64, _: f64, bounding_box: &mut Aabb) -> bool {
        std::mem::replace(
            bounding_box,
            Aabb::new(
                Vec3::with_values(self.x0, self.k - 0.0001, self.z0),
                Vec3::with_values(self.x1, self.k + 0.0001, self.z1),
            ),
        );
        true
    }
}

#[derive(Clone, Debug)]
pub struct RectSliceYz {
    material: Box<dyn Material>,
    y0: f64,
    y1: f64,
    z0: f64,
    z1: f64,
    k: f64,
}

impl RectSliceYz {
    pub fn new(material: Box<dyn Material>, params: (f64, f64, f64, f64, f64)) -> Self {
        let (y0, y1, z0, z1, k) = params;

        Self {
            material,
            y0,
            y1,
            z0,
            z1,
            k,
        }
    }
}

impl Hitable for RectSliceYz {
    fn hit(
        &self,
        r: &Ray,
        t_min: f64,
        t_max: f64,
        rec: &mut HitRecord,
    ) -> (bool, &Box<dyn Material>) {
        let t = (self.k - r.origin().x()) / r.direction().x();
        if t < t_min || t > t_max {
            return (false, self.get_material());
        }

        let y = r.origin().y() + t * r.direction().y();
        let z = r.origin().z() + t * r.direction().z();

        if z < self.z0 || z > self.z1 || y < self.y0 || y > self.y1 {
            return (false, self.get_material());
        }

        rec.u = (y - self.y0) / (self.y1 - self.y0);
        rec.v = (z - self.z0) / (self.z1 - self.z0);
        rec.t = t;
        rec.p = r.point_at_param(t);
        rec.normal = Vec3::with_values(1.0, 0.0, 0.0);

        (true, self.get_material())
    }

    fn get_material(&self) -> &Box<dyn Material> {
        &self.material
    }

    fn bounding_box(&self, _: f64, _: f64, bounding_box: &mut Aabb) -> bool {
        std::mem::replace(
            bounding_box,
            Aabb::new(
                Vec3::with_values(self.k - 0.0001, self.y0, self.z0),
                Vec3::with_values(self.k + 0.0001, self.y1, self.z1),
            ),
        );
        true
    }
}

#[derive(Clone, Debug)]
pub struct FlipNormals {
    child: Box<dyn Hitable>,
}

impl FlipNormals {
    pub fn new(child: Box<dyn Hitable>) -> Self {
        Self { child }
    }
}

impl Hitable for FlipNormals {
    fn hit(
        &self,
        r: &Ray,
        t_min: f64,
        t_max: f64,
        rec: &mut HitRecord,
    ) -> (bool, &Box<dyn Material>) {
        if self.child.hit(r, t_min, t_max, rec).0 {
            rec.normal = -rec.normal;
            return (true, self.child.get_material());
        } else {
            return (false, self.child.get_material());
        }
    }

    fn bounding_box(&self, t0: f64, t1: f64, bounding_box: &mut Aabb) -> bool {
        self.child.bounding_box(t0, t1, bounding_box)
    }

    fn get_material(&self) -> &Box<dyn Material> {
        self.child.get_material()
    }
}

#[derive(Clone, Debug)]
pub struct BoxObject {
    p0: Vec3,
    p1: Vec3,
    hitlist: HitableList,
}

impl BoxObject {
    pub fn new(p0: Vec3, p1: Vec3, material: Box<dyn Material>) -> Self {
        let mut hitlist = HitableList::new();
        hitlist.put(Box::new(RectSliceXy::new(
            dyn_clone::clone_box(&*material),
            (p0.x(), p1.x(), p0.y(), p1.y(), p1.z()),
        )));

        hitlist.put(Box::new(FlipNormals::new(Box::new(RectSliceXy::new(
            dyn_clone::clone_box(&*material),
            (p0.x(), p1.x(), p0.y(), p1.y(), p0.z()),
        )))));

        hitlist.put(Box::new(RectSliceXz::new(
            dyn_clone::clone_box(&*material),
            (p0.x(), p1.x(), p0.z(), p1.z(), p1.y()),
        )));

        hitlist.put(Box::new(FlipNormals::new(Box::new(RectSliceXz::new(
            dyn_clone::clone_box(&*material),
            (p0.x(), p1.x(), p0.z(), p1.z(), p0.y()),
        )))));

        hitlist.put(Box::new(RectSliceYz::new(
            dyn_clone::clone_box(&*material),
            (p0.y(), p1.y(), p0.z(), p1.z(), p1.x()),
        )));

        hitlist.put(Box::new(FlipNormals::new(Box::new(RectSliceYz::new(
            dyn_clone::clone_box(&*material),
            (p0.y(), p1.y(), p0.z(), p1.z(), p0.x()),
        )))));
        Self { p0, p1, hitlist }
    }
}

impl Hitable for BoxObject {
    fn hit(
        &self,
        r: &Ray,
        t_min: f64,
        t_max: f64,
        rec: &mut HitRecord,
    ) -> (bool, &Box<dyn Material>) {
        self.hitlist.hit(r, t_min, t_max, rec)
    }

    fn bounding_box(&self, _: f64, _: f64, bounding_box: &mut Aabb) -> bool {
        std::mem::replace(bounding_box, Aabb::new(self.p0, self.p1));
        true
    }

    fn get_material(&self) -> &Box<dyn Material> {
        self.hitlist.get_material()
    }
}
