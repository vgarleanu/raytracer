use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Clone)]
pub struct Camera {
    origin: Vec3,
    llc: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    uvw: (Vec3, Vec3, Vec3),
    lens_radius: f64,
}

impl Camera {
    pub fn new(
        lookfrom: Vec3,
        lookat: Vec3,
        vup: Vec3,
        vfov: f64,
        aspect: f64,
        aperture: f64,
        focus_dist: f64,
    ) -> Self {
        let theta = vfov * std::f64::consts::PI / 180.0;
        let half_height = (theta / 2.0).tan();
        let half_width = aspect * half_height;

        let w = (lookfrom - lookat).unit_vector();
        let u = vup.cross(w).unit_vector();
        let v = w.cross(u);

        Self {
            llc: lookfrom
                - half_width * focus_dist * u
                - half_height * focus_dist * v
                - focus_dist * w,
            horizontal: 2.0 * half_width * focus_dist * u,
            vertical: 2.0 * half_height * focus_dist * v,
            origin: lookfrom,
            lens_radius: aperture / 2.0,
            uvw: (u, v, w),
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        Ray::with_values(
            self.origin,
            self.llc + u * self.horizontal + v * self.vertical - self.origin,
        )
    }
}
