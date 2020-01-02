use crate::hitable::{HitRecord, Hitable, HitableList};
use crate::vec3::Vec3;
use rand::Rng;

// A, B, time
pub struct Ray(Vec3, Vec3, f64);

impl Ray {
    pub fn new() -> Self {
        Self(Vec3::new(), Vec3::new(), 0.0)
    }

    pub fn with_values(a: Vec3, b: Vec3, t: Option<f64>) -> Self {
        Self(a, b, t.unwrap_or(0.0))
    }

    pub fn origin(&self) -> Vec3 {
        self.0
    }

    pub fn direction(&self) -> Vec3 {
        self.1
    }

    pub fn point_at_param(&self, t: f64) -> Vec3 {
        self.0 + t * self.1
    }

    pub fn update(&mut self, rhs: Ray) {
        self.0 = rhs.origin();
        self.1 = rhs.direction();
    }

    pub fn time(&self) -> f64 {
        self.2
    }

    pub fn random_in_sphere() -> Vec3 {
        let mut rn = rand::thread_rng();
        loop {
            let p = 2.0 * Vec3::with_values(rn.gen::<f64>(), rn.gen::<f64>(), rn.gen::<f64>())
                - Vec3::with_values(1.0, 1.0, 1.0);
            if p.squared_len() < 1.0 {
                return p;
            }
        }
    }

    pub fn random_in_unit_disk() -> Vec3 {
        let mut rn = rand::thread_rng();
        loop {
            let p = 2.0 * Vec3::with_values(rn.gen::<f64>(), rn.gen::<f64>(), 0.0)
                - Vec3::with_values(1.0, 1.0, 0.0);
            if p.squared_len() < 1.0 {
                return p;
            }
        }
    }

    pub fn color(&self, world: &mut HitableList, depth: i64) -> Vec3 {
        let mut record = HitRecord::new();

        if world.hit(self, 0.001, std::f64::MAX, &mut record) {
            let mut scattered = Ray::new();
            let mut attenuation = Vec3::new();
            if depth < 50
                && record
                    .material
                    .scatter(self, &record, &mut attenuation, &mut scattered)
            {
                return attenuation * scattered.color(world, depth + 1);
            }
            return Vec3::with_values(0.0, 0.0, 0.0);
        } else {
            let unit_dir = self.direction().unit_vector();
            let t = 0.5 * (unit_dir.y() + 1.0);
            return (1.0 - t) * Vec3::with_values(1.0, 1.0, 1.0)
                + t * Vec3::with_values(0.5, 0.7, 1.0);
        }
    }
}
