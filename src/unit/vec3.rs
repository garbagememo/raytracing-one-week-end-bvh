
use std::fs;
use std::io::Write;
use std::ops::{Add, Mul, Rem, Sub};

#[derive(Copy, Clone, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub type Color = Vec3;

#[allow(dead_code)]
impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }
    pub fn zero() -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }
    pub fn mult(&self, b: &Vec3) -> Vec3 {
        Vec3::new(self.x * b.x, self.y * b.y, self.z * b.z)
    }
    pub fn norm(mut self) -> Vec3 {
        let l = 1.0 / (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        self.x = self.x * l;
        self.y = self.y * l;
        self.z = self.z * l;
        self
    }
    pub fn dot(&self, b: &Vec3) -> f64 {
        return self.x * b.x + self.y * b.y + self.z * b.z;
    }
    pub fn length(&self) -> f64 {
        return self.x * self.x + self.y * self.y + self.z * self.z;
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Self) -> Self {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Self) -> Self {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f64) -> Self {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Rem for Vec3 {
    type Output = Vec3;
    fn rem(self, rhs: Self) -> Self {
        Vec3::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }
}

fn clamp(x: f64) -> f64 {
    if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}

fn to_int(x: f64) -> u8 {
    (clamp(x).powf(1.0 / 2.2) * 255.0 + 0.5) as u8
}

#[allow(dead_code)]
fn save_ppm_file(filename: &str, image: Vec<Color>, width: usize, height: usize) {
    let mut f = fs::File::create(filename).unwrap();
    
    writeln!(f, "P3\n{} {}\n{}", width, height, 255).unwrap();
    for i in 0..(width * (height)) {
        write!(
            f,
            "{} {} {} ",
            to_int(image[i as usize].x),
            to_int(image[i as usize].y),
            to_int(image[i as usize].z)
        )
        .unwrap();
    }
}

pub fn save_png_file(filename: &str, out_image: Vec<Color>, width: usize, height: usize) {
    let mut imgbuf = image::ImageBuffer::new(width as u32, height as u32);

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let i: usize = (x as usize) + (y as usize) * width;
        let r = to_int(out_image[i].x);
        let g = to_int(out_image[i].y);
        let b = to_int(out_image[i].z);
        *pixel = image::Rgb([r, g, b]);
    }

    // Save the image as “fractal.png”, the format is deduced from the path
    imgbuf.save(filename).unwrap();
}
