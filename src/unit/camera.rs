use crate::rayunit::*;
use rand::Rng;
use rayon::prelude::*;

pub struct Camera {
    origin: Vec3,
    corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    cu: Vec3,
    cv: Vec3,
    lens_radius: f64,
    image_height: u64,
    image_width: u64,
    samples_per_pixel: u64,
    max_depth: u64,
}

impl Camera {
    pub fn new(
        lookfrom: Vec3,
        lookat: Vec3,
        vup: Vec3,
        vfov: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
        image_height: u64,
        image_width: u64,
        samples_per_pixel: u64,
        max_depth: u64,
    ) -> Camera {
        let theta = std::f64::consts::PI / 180.0 * vfov;
        let viewport_height: f64 = 2.0 * (theta / 2.0).tan();
        let viewport_width: f64 = aspect_ratio * viewport_height;

        let cw = (lookfrom - lookat).unit_vector();
        let cu = vup.cross(cw).unit_vector();
        let cv = cw.cross(cu);

        let h = cu * viewport_width * focus_dist;
        let v = cv * viewport_height * focus_dist;

        let llc = lookfrom - h / 2.0 - v / 2.0 - cw * focus_dist;

        Camera {
            origin: lookfrom,
            corner: llc,
            horizontal: h,
            vertical: v,
            cu,
            cv,
            lens_radius: aperture / 2.0,
            image_height,
            image_width,
            samples_per_pixel,
            max_depth,
        }
    }

    fn get_ray(&self, u: f64, v: f64) -> Ray {
        let rd = Vec3::random_in_unit_sphere() * self.lens_radius;
        let offset = self.cu * rd.x() + self.cv * rd.y();

        return Ray::new(
            self.origin + offset,
            (self.corner + (self.horizontal * u)) + (self.vertical * v) - self.origin,
        );
    }

    fn ray_color(&self, r: &Ray, world: &Tree, depth: u64) -> Color {
        if depth <= 0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        if let Some(rec) = world.hit(r, 0.001, f64::INFINITY) {
            if let Some((attentuation, scattered)) = rec.mat.scatter(r, &rec) {
                return self.ray_color(&scattered, world, depth - 1) * attentuation;
            }
        }

        let unit_direction = r.direction().unit_vector();
        let a = 0.5 * (unit_direction.y() + 1.0);
        Color::new(
            (1.0 - a) + (a * 0.5),
            (1.0 - a) + (a * 0.7),
            (1.0 - a) + (a * 1.0),
        )
    }

    pub fn render(&self, world: &Tree) -> Vec<u8> {
        let mut full_image: Vec<u8> = Vec::new();
        for j in (0..self.image_height).rev() {
            let scanline: Vec<Color> = (0..self.image_width)
                .into_par_iter()
                .map(|i| {
                    let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                    for _ in 0..self.samples_per_pixel {
                        let mut rng = rand::thread_rng();
                        let random_u: f64 = rng.gen();
                        let random_v: f64 = rng.gen();

                        let u = ((i as f64) + random_u) / ((self.image_width - 1) as f64);
                        let v = ((j as f64) + random_v) / ((self.image_height - 1) as f64);
                        let r = self.get_ray(u, v);
                        pixel_color += self.ray_color(&r, &world, self.max_depth);
                    }
                    pixel_color
                })
                .collect();

            let flattened: Vec<u8> = scanline
                .iter()
                .flat_map(|color| {
                    vec![
                        color.r(self.samples_per_pixel),
                        color.g(self.samples_per_pixel),
                        color.b(self.samples_per_pixel),
                    ]
                })
                .collect();
            full_image.extend(flattened);
        }
        full_image
    }
}
