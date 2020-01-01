use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Clone)]
pub struct Camera {
    origin: Vec3,
    llc: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            llc: Vec3::with_values(-2.0, -1.0, -1.0),
            horizontal: Vec3::with_values(4.0, 0.0, 0.0),
            vertical: Vec3::with_values(0.0, 2.0, 0.0),
            origin: Vec3::with_values(0.0, 0.0, 0.0),
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        Ray::with_values(
            self.origin,
            self.llc + u * self.horizontal + v * self.vertical - self.origin,
        )
    }
}
