use crate::ray::Ray;
use crate::vec3::Vec3;

/// Struct describes a minimal axis aligned bounding box, use withink the ray tracing algorithm to
/// detect ray intersection
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
            let t0 = ((self.min[i] - ray.origin()[i]) / ray.direction()[i])
                .min((self.max[i] - ray.origin()[i]) / ray.direction()[i]);

            let t1 = ((self.min[i] - ray.origin()[i]) / ray.direction()[i])
                .max((self.max[i] - ray.origin()[i]) / ray.direction()[i]);

            tmin = t0.max(tmin);
            tmax = t1.min(tmax);

            if tmax <= tmin {
                return false;
            }
        }
        true
    }
}
