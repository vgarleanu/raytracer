use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Clone, Copy)]
pub struct Vec3(f64, f64, f64);

impl Vec3 {
    pub fn new() -> Self {
        Self(0.0, 0.0, 0.0)
    }

    pub fn with_values(x: f64, y: f64, z: f64) -> Self {
        Self(x, y, z)
    }

    pub fn x(&self) -> f64 {
        self.0
    }

    pub fn y(&self) -> f64 {
        self.1
    }

    pub fn z(&self) -> f64 {
        self.2
    }

    pub fn r(&self) -> f64 {
        self.0
    }

    pub fn g(&self) -> f64 {
        self.1
    }

    pub fn b(&self) -> f64 {
        self.2
    }

    pub fn dot(&self, rhs: Vec3) -> f64 {
        self.0 * rhs.x() + self.1 * rhs.y() + self.2 * rhs.z()
    }

    pub fn cross(&self, rhs: Vec3) -> Self {
        let x = self.1 * rhs.z() - self.2 * rhs.y();
        let y = -(self.0 * rhs.z() - self.2 * rhs.x());
        let z = self.0 * rhs.y() - self.1 * rhs.x();

        Self::with_values(x, y, z)
    }

    pub fn len(&self) -> f64 {
        self.squared_len().sqrt()
    }

    pub fn squared_len(&self) -> f64 {
        self.0 * self.0 + self.1 * self.1 + self.2 * self.2
    }

    pub fn unit_vector(&self) -> Self {
        *self / self.len()
    }

    pub fn make_unit_vector(&mut self) {
        let k = 1.0 / self.len();
        self.0 *= k;
        self.1 *= k;
        self.2 *= k;
    }

    pub fn update(&mut self, rhs: Vec3) {
        self.0 = rhs.x();
        self.1 = rhs.y();
        self.2 = rhs.z();
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3::with_values(self.0 + rhs.x(), self.1 + rhs.y(), self.2 + rhs.z())
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
        *self = Vec3::with_values(self.0 + rhs.x(), self.1 + rhs.y(), self.2 + rhs.z());
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3::with_values(self.0 - rhs.x(), self.1 - rhs.y(), self.2 - rhs.z())
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Vec3) {
        *self = Vec3::with_values(self.0 - rhs.x(), self.1 - rhs.y(), self.2 - rhs.z());
    }
}

impl Mul<Vec3> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::with_values(self.0 * rhs.x(), self.1 * rhs.y(), self.2 * rhs.z())
    }
}

impl MulAssign<Vec3> for Vec3 {
    fn mul_assign(&mut self, rhs: Vec3) {
        *self = Vec3::with_values(self.0 * rhs.x(), self.1 * rhs.y(), self.2 * rhs.z());
    }
}

impl Div<Vec3> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: Vec3) -> Vec3 {
        Vec3::with_values(self.0 / rhs.x(), self.1 / rhs.y(), self.2 / rhs.z())
    }
}

impl DivAssign<Vec3> for Vec3 {
    fn div_assign(&mut self, rhs: Vec3) {
        *self = Vec3::with_values(self.0 / rhs.x(), self.1 / rhs.y(), self.2 / rhs.z());
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Vec3 {
        Vec3::with_values(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        *self = Vec3::with_values(self.0 * rhs, self.1 * rhs, self.2 * rhs);
    }
}

impl Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f64) -> Vec3 {
        Vec3::with_values(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}

impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, rhs: f64) {
        *self = Vec3::with_values(self.0 / rhs, self.1 / rhs, self.2 / rhs);
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, lhs: Vec3) -> Vec3 {
        Vec3::with_values(self * lhs.0, self * lhs.1, self * lhs.2)
    }
}
