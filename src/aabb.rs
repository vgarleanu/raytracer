use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Clone, Debug)]
pub struct Aabb {
    min: Vec3,
    max: Vec3,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Aabb { min, max }
    }

    pub fn min(&self) -> Vec3 {
        self.min
    }

    pub fn max(&self) -> Vec3 {
        self.max
    }

    pub fn hit(&self, ray: &Ray, mut tmin: f64, mut tmax: f64) -> bool {
        for i in 0..3 {
            let inverse_d = 1.0 / ray.direction()[i];

            let mut t0 = (self.min[i] - ray.origin()[i]) * inverse_d;
            let mut t1 = (self.max[i] - ray.origin()[i]) * inverse_d;

            if inverse_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }

            tmin = if t0 > tmin { t0 } else { tmin };
            tmax = if t1 < tmax { t1 } else { tmax };

            if tmax <= tmin {
                return false;
            }
        }
        true
    }
}
