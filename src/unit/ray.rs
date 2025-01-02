use crate::rayunit::*;

#[derive(Copy, Clone)]
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub fn new(o: Vec3, dir: Vec3) -> Ray {
        Ray {
            origin: o,
            direction: dir,
        }
    }
    pub fn origin(self) -> Vec3 {
        self.origin
    }
    pub fn direction(self) -> Vec3 {
        self.direction
    }
    pub fn at(self, t: f64) -> Vec3 {
        return self.origin + (self.direction * t);
    }
}
