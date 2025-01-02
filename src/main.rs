mod raymod;
use raymod::*;

use rayon::prelude::*;

use std::io::Write;

fn radiance(r: &Ray, depth: u8,	scene: &Scene) -> Vec3 {
    let ir=scene.intersect(r);
    if ir.b==false {
        return Vec3::zero();
    }
    let obj = &scene.objects[ir.id];
    let x = r.o + r.d * ir.t;
    let n = (x - obj.p).norm();
    let nl = if n.dot(&r.d) < 0.0 { n } else { n * -1.0 };
    let mut f = obj.c;
    let p = if f.x > f.y && f.x > f.z {
        f.x
    } else if f.y > f.z {
        f.y
    } else {
        f.z
    };
    let depth = depth + 1;
    if depth > 5 {
        if depth < 127 && random() < p {
            f = f * (1.0 / p);
        } else {
            return obj.e;
        }
    }

    return match obj.refl {
        Refl::Diff => {
            let r1 = 2.0 * std::f64::consts::PI * random();
            let r2 = random();
            let r2s = r2.sqrt();
            let w = nl;
            let u = ((if w.x.abs() > 0.1 {
                Vec3::new(0.0, 1.0, 0.0)
            } else {
                Vec3::new(1.0, 0.0, 0.0)
            }) % w)
                .norm();
            let v = w % u;
            let d =
                (u * f64::cos(r1) * r2s + v * f64::sin(r1) * r2s + w * (1.0 - r2).sqrt()).norm();
            obj.e + f.mult(&radiance(&Ray::new(x, d), depth,scene))
        }
        Refl::Spec => obj.e + f.mult(&radiance(&Ray::new(x, r.d - n * 2.0 * n.dot(&r.d)), depth,scene)),
        _ => {
            // Refl.Refr
            let refl_ray = Ray::new(x, r.d - n * 2.0 * n.dot(&r.d));
            let into = n.dot(&nl) > 0.0;
            let nc = 1.0;
            let nt = 1.5;
            let nnt = if into { nc / nt } else { nt / nc };
            let ddn = r.d.dot(&nl);
            let cos2t = 1.0 - nnt * nnt * (1.0 - ddn * ddn);
            if cos2t < 0.0 {
                obj.e + f.mult(&radiance(&refl_ray, depth,scene))
            } else {
                let tdir =
                    r.d * nnt - n * ((if into { 1.0 } else { -1.0 }) * (ddn * nnt + cos2t.sqrt()));
                tdir.norm();
                let a = nt - nc;
                let b = nt + nc;
                let r0 = a * a / (b * b);
                let c = 1.0 - (if into { -ddn } else { tdir.dot(&n) });
                let re = r0 + (1.0 - r0) * c * c * c * c * c;
                let tr = 1.0 - re;
                let p = 0.25 + 0.5 * re;
                let rp = re / p;
                let tp = tr / (1.0 - p);
                obj.e
                    + f.mult(
                        &(if depth > 2 {
                            if random() < p {
                                radiance(&refl_ray, depth,scene) * rp
                            } else {
                                radiance(&Ray::new(x, tdir), depth,scene) * tp
                            }
                        } else {
                            radiance(&refl_ray, depth,scene) * re
                                + radiance(&Ray::new(x, tdir), depth,scene) * tr
                        }),
                    )
            }
        }
    };
}

fn main() {

    let args = parameters();
    println!("{:?}", args);
    
	let mut scene=Scene::init();
    match args.m{
        0=> scene.model_init0(),
        1=> scene.model_init1(),
        2=> scene.model_init2(),
        3=> scene.model_init3(),
        4=> scene.model_init4(),
        5=> scene.model_init5(),
        6=> scene.model_init6(),
        7=> scene.model_init7(),
        8=> scene.model_init8(),
        9=> scene.model_init9(),
        _=> scene.model_init0(),
    };

	
    let w: usize = args.w;
    let h: usize = (480.0*w as f64/640.0) as usize;
    let samps = args.s;
    println!("samps={}",samps);

    let cam = Ray::new(
        Vec3::new(50.0, 52.0, 295.6),
        Vec3::new(0.0, -0.042612, -1.0).norm(),
    );

    let cx = Vec3::new((w as f64) * 0.5135 / (h as f64), 0.0, 0.0);
    let cy = (cx % cam.d).norm() * 0.5135;
    let mut image = vec![Color::zero(); (w * h) as usize];

    let bands: Vec<(usize, &mut [Color])> = image.chunks_mut(w as usize).enumerate().collect();
    bands.into_par_iter().for_each(|(y, band)| {
        let y2 = h - (y as usize) - 1;
        if (y % 10) == 0 {
            writeln!(
                std::io::stderr(),
                "Rendering ({} spp) {:5.2}%",
                samps * 4,
                100.0 * (y as f64) / ((h as f64) - 1.0)
            )
            .unwrap();
        }
        for x in 0..w {
            let mut r = Vec3::zero();
            for sy in 0..2 {
                for sx in 0..2 {
                    for _s in 0..samps {
                        let r1 = 2.0 * random();
                        let dx = if r1 < 1.0 {
                            r1.sqrt() - 1.0
                        } else {
                            1.0 - (2.0 - r1).sqrt()
                        };
                        let r2 = 2.0 * random();
                        let dy = if r2 < 1.0 {
                            r2.sqrt() - 1.0
                        } else {
                            1.0 - (2.0 - r2).sqrt()
                        };
                        let d = cx
                            * ((((sx as f64) + 0.5 + dx) / 2.0 + (x as f64)) / (w as f64) - 0.5)
                            + cy * ((((sy as f64) + 0.5 + dy) / 2.0 + (y2 as f64)) / (h as f64)
                                - 0.5)
                            + cam.d;
                        r = r + radiance(&(Ray::new(cam.o + d * 140.0, d.norm())), 0,&scene)
                            * (1.0 / (samps as f64));
                    }
                    band[x as usize] = band[x as usize] + r * (1.0 / 4.0 as f64);
                    r = Vec3::zero();
                }
            }
        }
    });

    //    save_ppm_file("image.ppm", image, w, h);
    save_png_file(&args.output, image, w, h);
}
