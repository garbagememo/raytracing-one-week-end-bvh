use crate::rayunit::*;
use std::sync::Arc;

pub struct HitRecord {
    pub p: Vec3,
    pub normal: Vec3,
    pub mat: Arc<dyn Scatter>,
    pub t: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        self.front_face = r.direction().dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            outward_normal * -1.0
        };
    }
}

pub trait Hit: Send + Sync {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

#[derive(Clone)]
pub struct Sphere {
    center: Vec3,
    radius: f64,
    mat: Arc<dyn Scatter>,
    movement: Vec3,
}

impl Sphere {
    pub fn new(c: Vec3, r: f64, m: Arc<dyn Scatter>, d: Vec3) -> Sphere {
        Sphere {
            center: c,
            radius: r,
            mat: m,
            movement: d,
        }
    }

    pub fn lower_bound(&self) -> Vec3 {
        let r = self.radius;
        self.center - Vec3::new(r, r, r)
    }

    pub fn upper_bound(&self) -> Vec3 {
        let r = self.radius;
        self.center + Vec3::new(r, r, r)
    }

    pub fn step_frame(&self, time_delta: f64) -> Sphere {
        Sphere::new(
            self.center + (self.movement * time_delta),
            self.radius,
            self.mat.clone(),
            self.movement,
        )
    }
}

impl Hit for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // Do we get hit?
        let oc = r.origin() - self.center;
        let a = r.direction().length().powi(2);
        let half_b = oc.dot(r.direction());
        let c = oc.length().powi(2) - self.radius * self.radius;

        let discriminant = half_b.powi(2) - a * c;
        if discriminant < 0.0 {
            return None;
        }

        // Find the nearest root in the range
        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let point = r.at(root);
        let mut rec = HitRecord {
            t: root,
            p: point,
            mat: self.mat.clone(),
            normal: Vec3::new(0.0, 0.0, 0.0),
            front_face: false,
        };

        let outward_normal = (point - self.center) / self.radius;
        rec.set_face_normal(r, outward_normal);

        Some(rec)
    }
}
