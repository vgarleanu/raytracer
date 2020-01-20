use crate::hitable::{HitRecord, Hitable, HitableList};
use crate::vec3::Vec3;
use rand::Rng;

// A, B, time
pub struct Ray(Vec3, Vec3, f64, pub bool);

impl Ray {
    pub fn new(debug: bool) -> Self {
        Self(Vec3::new(), Vec3::new(), 0.0, debug)
    }

    pub fn with_values(a: Vec3, b: Vec3, t: Option<f64>, debug: bool) -> Self {
        Self(a, b, t.unwrap_or(0.0), debug)
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

        let hit = world.hit(self, 0.001, std::f64::MAX, &mut record);
        if hit.0 {
            let mut scattered = Ray::new(self.3);
            let mut attenuation = Vec3::new();
            let emitted = hit.1.emitted(record.u, record.v, record.p);
            if depth < 50
                && hit
                    .1
                    .scatter(self, &record, &mut attenuation, &mut scattered)
            {
                let color = scattered.color(world, depth + 1);
                return emitted + (attenuation * color);
            }
            return emitted;
        }
        if self.3 {
            Vec3::with_values(1.0, 1.0, 1.0)
        } else {
            Vec3::new()
        }
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self::new(false)
    }
}
