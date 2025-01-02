use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub};

use rand::Rng;

#[derive(Copy, Clone)]
pub struct Vec3 {
    pub values: [f64; 3],
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { values: [x, y, z] }
    }

    pub fn x(&self) -> f64 {
        self.values[0]
    }
    pub fn y(&self) -> f64 {
        self.values[1]
    }
    pub fn z(&self) -> f64 {
        self.values[2]
    }

    pub fn negative(&self) -> Vec3 {
        Vec3 {
            values: [-self.x(), -self.y(), -self.z()],
        }
    }

    pub fn unit_vector(&self) -> Vec3 {
        *self / self.length()
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3 {
            values: ([
                self.y() * other.z() - self.z() * other.y(),
                self.z() * other.x() - self.x() * other.z(),
                self.x() * other.y() - self.y() * other.x(),
            ]),
        }
    }

    pub fn dot(&self, other: Vec3) -> f64 {
        self.x() * other.x() + self.y() * other.y() + self.z() * other.z()
    }

    pub fn length_squared(&self) -> f64 {
        self.x() * self.x() + self.y() * self.y() + self.z() * self.z()
    }

    pub fn random(lower: f64, upper: f64) -> Vec3 {
        let mut rng = rand::thread_rng();
        Vec3 {
            values: [
                rng.gen_range(lower..upper),
                rng.gen_range(lower..upper),
                rng.gen_range(lower..upper),
            ],
        }
    }

    pub fn random_in_unit_sphere() -> Vec3 {
        loop {
            let v = Vec3::random(-1.0, 1.0);
            if v.length() < 1.0 {
                return v;
            }
        }
    }

    pub fn near_zero(self) -> bool {
        const TINY_VALUE: f64 = 1.0e-8;
        self.x().abs() < TINY_VALUE && self.y().abs() < TINY_VALUE && self.z().abs() < TINY_VALUE
    }

    pub fn reflect(self, n: Vec3) -> Vec3 {
        self - n * 2.0 * self.dot(n)
    }

    pub fn refract(self, n: Vec3, etai_over_etat: f64) -> Vec3 {
        let cos_theta = (self * (-1.0)).dot(n).min(1.0);
        let r_out_perp = (self + (n * cos_theta)) * etai_over_etat;
        let r_out_parallel = n * -((1.0 - r_out_perp.length_squared().abs()).sqrt());

        r_out_perp + r_out_parallel
    }

    pub fn min(a: Vec3, b: Vec3) -> Vec3 {
        Vec3::new(a.x().min(b.x()), a.y().min(b.y()), a.z().min(b.z()))
    }

    pub fn max(a: Vec3, b: Vec3) -> Vec3 {
        Vec3::new(a.x().max(b.x()), a.y().max(b.y()), a.z().max(b.z()))
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            values: [
                self.x() + other.x(),
                self.y() + other.y(),
                self.z() + other.z(),
            ],
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        *self = Vec3 {
            values: [
                self.x() + other.x(),
                self.y() + other.y(),
                self.z() + other.z(),
            ],
        }
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, other: f64) -> Vec3 {
        Vec3 {
            values: [self.x() * other, self.y() * other, self.z() * other],
        }
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, other: f64) {
        *self = Vec3 {
            values: [self.x() * other, self.y() * other, self.z() * other],
        }
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            values: [
                self.x() - other.x(),
                self.y() - other.y(),
                self.z() - other.z(),
            ],
        }
    }
}

impl Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, other: f64) -> Self::Output {
        Vec3 {
            values: [self.x() / other, self.y() / other, self.z() / other],
        }
    }
}

impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, other: f64) {
        *self = Vec3 {
            values: [self.x() / other, self.y() / other, self.z() / other],
        }
    }
}

#[derive(Copy, Clone)]
pub struct Color {
    values: [f64; 3],
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Color { values: [r, g, b] }
    }

    pub fn format_color(self, samples_per_pixel: u64) -> String {
        let ir = (256.0
            * (self.values[0] / (samples_per_pixel as f64))
                .sqrt()
                .clamp(0.0, 0.999)) as u64;
        let ig = (256.0
            * (self.values[1] / (samples_per_pixel as f64))
                .sqrt()
                .clamp(0.0, 0.999)) as u64;
        let ib = (256.0
            * (self.values[2] / (samples_per_pixel as f64))
                .sqrt()
                .clamp(0.0, 0.999)) as u64;

        format!("{} {} {}", ir, ig, ib)
    }

    pub fn r(self, samples_per_pixel: u64) -> u8 {
        let ir: u8 = (256.0
            * (self.values[0] / (samples_per_pixel as f64))
                .sqrt()
                .clamp(0.0, 0.999)) as u8;
        ir
    }

    pub fn g(self, samples_per_pixel: u64) -> u8 {
        let ig: u8 = (256.0
            * (self.values[1] / (samples_per_pixel as f64))
                .sqrt()
                .clamp(0.0, 0.999)) as u8;
        ig
    }

    pub fn b(self, samples_per_pixel: u64) -> u8 {
        let ib: u8 = (256.0
            * (self.values[2] / (samples_per_pixel as f64))
                .sqrt()
                .clamp(0.0, 0.999)) as u8;
        ib
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, other: Self) {
        *self = Color {
            values: [
                self.values[0] + other.values[0],
                self.values[1] + other.values[1],
                self.values[2] + other.values[2],
            ],
        }
    }
}

impl Mul<f64> for Color {
    type Output = Color;

    fn mul(self, other: f64) -> Self::Output {
        Color::new(
            self.values[0] * other,
            self.values[1] * other,
            self.values[2] * other,
        )
    }
}

impl Mul for Color {
    type Output = Color;
    fn mul(self, other: Self) -> Self::Output {
        Color::new(
            self.values[0] * other.values[0],
            self.values[1] * other.values[1],
            self.values[2] * other.values[2],
        )
    }
}
