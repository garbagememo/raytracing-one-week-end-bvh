mod rayunit;
use rayunit::*;

use rand::Rng;
use std::fs;
use std::io::Write;
use std::sync::Arc;

fn random_scene() -> Tree {
    let mut rng = rand::thread_rng();
    let mut world = Tree::new(1);

    let ground_mat = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    let ground_sphere = Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_mat,
        Vec3::new(0.0, 0.0, 0.0),
    );

    world.push(Box::new(ground_sphere));

    for a in 0..25 {
        for b in 0..25 {
            let choose_mat: f64 = rng.gen();
            let center = Vec3::new(
                (a as f64) + rng.gen_range(0.0..0.9),
                0.2,
                (b as f64) + rng.gen_range(0.0..0.9),
            );
            let movement = Vec3::new(
                (a as f64) + rng.gen_range(0.0..0.9),
                0.2,
                (b as f64) + rng.gen_range(0.0..0.9),
            );

            if choose_mat < 0.8 {
                // Diffuse
                let r = rng.gen_range(0.0..1.0);
                let b = rng.gen_range(0.0..1.0);
                let g = rng.gen_range(0.0..1.0);
                let albedo = Color::new(r, g, b);
                let sphere_mat = Arc::new(Lambertian::new(albedo));
                let sphere = Sphere::new(center, 0.2, sphere_mat, movement);

                world.push(Box::new(sphere));
            } else if choose_mat < 0.95 {
                // Metal
                let r = rng.gen_range(0.4..1.0);
                let b = rng.gen_range(0.4..1.0);
                let g = rng.gen_range(0.4..1.0);
                let albedo = Color::new(r, g, b);
                let fuzz = rng.gen_range(0.0..0.5);
                let sphere_mat = Arc::new(Metal::new(albedo, fuzz));
                let sphere = Sphere::new(center, 0.2, sphere_mat, movement);

                world.push(Box::new(sphere));
            } else {
                // Glass
                let sphere_mat = Arc::new(Dielectric::new(1.5));
                let sphere = Sphere::new(center, 0.2, sphere_mat, movement);

                world.push(Box::new(sphere));
            }
        }
    }

    let mat1 = Arc::new(Dielectric::new(1.5));
    let mat2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    let mat3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));

    let sphere1 = Sphere::new(
        Vec3::new(0.0, 1.0, 0.0),
        1.0,
        mat1,
        Vec3::new(0.0, 0.1, 0.0),
    );
    let sphere2 = Sphere::new(
        Vec3::new(-4.0, 1.0, 0.0),
        1.0,
        mat2,
        Vec3::new(0.0, 0.1, 0.0),
    );
    let sphere3 = Sphere::new(
        Vec3::new(4.0, 1.0, 0.0),
        1.0,
        mat3,
        Vec3::new(0.0, 0.1, 0.0),
    );

    world.push(Box::new(sphere1));
    world.push(Box::new(sphere2));
    world.push(Box::new(sphere3));

    world
}

#[allow(dead_code)]
fn save_ppm_file(filename: &str, image: Vec<u8>, width: usize, height: usize) {
    let mut f = fs::File::create(filename).unwrap();

    writeln!(f, "P3\n{} {}\n{}", width, height, 255).unwrap();
    for i in 0..(width * (height)) {
        write!(
            f,
            "{} {} {} ",
            image[i * 3 as usize],
            image[i * 3 + 1 as usize],
            image[i * 3 + 2 as usize]
        )
        .unwrap();
    }
}

#[allow(dead_code)]
pub fn save_png_file(filename: &str, out_image: Vec<u8>, width: usize, height: usize) {
    let mut imgbuf = image::ImageBuffer::new(width as u32, height as u32);

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let i: usize = (x as usize) + (y as usize) * width;
        let r = out_image[i * 3];
        let g = out_image[i * 3 + 1];
        let b = out_image[i * 3 + 2];
        *pixel = image::Rgb([r, g, b]);
    }

    // Save the image as “fractal.png”, the format is deduced from the path
    imgbuf.save(filename).unwrap();
}

fn main() {
    // Image
    const ASPECT_RATIO: f64 = 3.0 / 2.0;
    const IMAGE_WIDTH: usize = 600;
    const IMAGE_HEIGHT: usize = ((IMAGE_WIDTH as f64) / ASPECT_RATIO) as usize;
    const SAMPLES_PER_PIXEL: u64 = 50;
    const MAX_DEPTH: u64 = 20;

    // World
    let mut world = random_scene();

    // Camera
    let lookfrom = Vec3::new(13.0, 2.0, 3.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.0;

    let cam = Camera::new(
        lookfrom,
        lookat,
        vup,
        20.0,
        ASPECT_RATIO,
        aperture,
        dist_to_focus,
        IMAGE_HEIGHT as u64,
        IMAGE_WIDTH as u64,
        SAMPLES_PER_PIXEL,
        MAX_DEPTH,
    );

    // render
    let pixels = cam.render(&world);
    println!("pixels.len {}", pixels.len());
    //    save_ppm_file("test.ppm",pixels,IMAGE_WIDTH,IMAGE_HEIGHT);

    save_png_file("test.png", pixels, IMAGE_WIDTH, IMAGE_HEIGHT);
}
