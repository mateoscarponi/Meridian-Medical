#![cfg_attr(not(feature = "hydrate"), allow(dead_code))]
//! A procedurally modeled human spine (7 cervical, 12 thoracic, 5 lumbar vertebrae + sacrum),
//! rotating slowly in real time — rendered with raw WebGL2 from Rust/WebAssembly.
//!
//! This replaces react-three-fiber + three.js. Geometry builders (lathe, torus, cylinder, capsule,
//! sphere), the centripetal Catmull-Rom spine curve, scene transforms, lighting
//! (ambient + hemisphere + 2 directional + point) and the MeshStandardMaterial shading
//! (GGX specular, Lambert diffuse, ACES filmic tone mapping, sRGB output) are reproduced from
//! three.js so the model looks the same.
//!
//! Optimization: the scene below the animated groups never changes, so all ~200 meshes are baked
//! once into two vertex buffers (bone + disc). Each frame is 2 draw calls instead of ~200.

use leptos::html::{Canvas, Div};
use leptos::prelude::*;
use std::f64::consts::PI;

// ---------------------------------------------------------------- math (column-major, like three)

type V3 = [f64; 3];

#[derive(Clone, Copy, Debug)]
struct M4([f64; 16]);

impl M4 {
    fn identity() -> Self {
        M4([1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.])
    }
    fn mul(&self, b: &M4) -> M4 {
        let a = &self.0;
        let b = &b.0;
        let mut r = [0.0; 16];
        for c in 0..4 {
            for row in 0..4 {
                r[c * 4 + row] = (0..4).map(|k| a[k * 4 + row] * b[c * 4 + k]).sum();
            }
        }
        M4(r)
    }
    fn translate(x: f64, y: f64, z: f64) -> M4 {
        let mut m = M4::identity();
        m.0[12] = x;
        m.0[13] = y;
        m.0[14] = z;
        m
    }
    fn scale(x: f64, y: f64, z: f64) -> M4 {
        let mut m = M4::identity();
        m.0[0] = x;
        m.0[5] = y;
        m.0[10] = z;
        m
    }
    /// three.js `makeRotationFromEuler` with the default 'XYZ' order.
    fn euler(x: f64, y: f64, z: f64) -> M4 {
        let (a, b) = (x.cos(), x.sin());
        let (c, d) = (y.cos(), y.sin());
        let (e, f) = (z.cos(), z.sin());
        let (ae, af, be, bf) = (a * e, a * f, b * e, b * f);
        let mut t = M4::identity().0;
        t[0] = c * e;
        t[4] = -c * f;
        t[8] = d;
        t[1] = af + be * d;
        t[5] = ae - bf * d;
        t[9] = -b * c;
        t[2] = bf - ae * d;
        t[6] = be + af * d;
        t[10] = a * c;
        M4(t)
    }
    fn quat(q: [f64; 4]) -> M4 {
        let [x, y, z, w] = q;
        let (x2, y2, z2) = (x + x, y + y, z + z);
        let (xx, xy, xz) = (x * x2, x * y2, x * z2);
        let (yy, yz, zz) = (y * y2, y * z2, z * z2);
        let (wx, wy, wz) = (w * x2, w * y2, w * z2);
        let mut t = M4::identity().0;
        t[0] = 1. - (yy + zz);
        t[1] = xy + wz;
        t[2] = xz - wy;
        t[4] = xy - wz;
        t[5] = 1. - (xx + zz);
        t[6] = yz + wx;
        t[8] = xz + wy;
        t[9] = yz - wx;
        t[10] = 1. - (xx + yy);
        M4(t)
    }
    fn point(&self, p: V3) -> V3 {
        let m = &self.0;
        [
            m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12],
            m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13],
            m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14],
        ]
    }
    /// Normal matrix (inverse-transpose of the upper 3×3), applied and normalized.
    fn normal(&self, n: V3) -> V3 {
        let m = &self.0;
        let (a, b, c) = (m[0], m[4], m[8]);
        let (d, e, f) = (m[1], m[5], m[9]);
        let (g, h, i) = (m[2], m[6], m[10]);
        // cofactor matrix == det * inverse-transpose; the scale cancels on normalize
        let co = [
            [e * i - f * h, -(d * i - f * g), d * h - e * g],
            [-(b * i - c * h), a * i - c * g, -(a * h - b * g)],
            [b * f - c * e, -(a * f - c * d), a * e - b * d],
        ];
        let det = a * co[0][0] + b * co[0][1] + c * co[0][2];
        let s = if det < 0.0 { -1.0 } else { 1.0 };
        let r = [
            s * (co[0][0] * n[0] + co[1][0] * n[1] + co[2][0] * n[2]),
            s * (co[0][1] * n[0] + co[1][1] * n[1] + co[2][1] * n[2]),
            s * (co[0][2] * n[0] + co[1][2] * n[1] + co[2][2] * n[2]),
        ];
        norm(r)
    }
    fn to_f32(self) -> [f32; 16] {
        self.0.map(|x| x as f32)
    }
}

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn len(a: V3) -> f64 {
    dot(a, a).sqrt()
}
fn norm(a: V3) -> V3 {
    let l = len(a);
    if l == 0.0 {
        a
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}

/// `Quaternion.setFromUnitVectors(from, to)`
fn quat_from_units(from: V3, to: V3) -> [f64; 4] {
    let r = dot(from, to) + 1.0;
    let q = if r < f64::EPSILON {
        if from[0].abs() > from[2].abs() {
            [-from[1], from[0], 0.0, 0.0]
        } else {
            [0.0, -from[2], from[1], 0.0]
        }
    } else {
        let c = cross(from, to);
        [c[0], c[1], c[2], r]
    };
    let l = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    q.map(|v| v / l)
}

// ---------------------------------------------------------------- curve (three.js CatmullRomCurve3, centripetal)

struct Curve {
    pts: Vec<V3>,
    lengths: Vec<f64>,
}

fn cubic(x0: f64, x1: f64, x2: f64, x3: f64, dt0: f64, dt1: f64, dt2: f64, t: f64) -> f64 {
    let mut t1 = (x1 - x0) / dt0 - (x2 - x0) / (dt0 + dt1) + (x2 - x1) / dt1;
    let mut t2 = (x2 - x1) / dt1 - (x3 - x1) / (dt1 + dt2) + (x3 - x2) / dt2;
    t1 *= dt1;
    t2 *= dt1;
    let (c0, c1) = (x1, t1);
    let c2 = -3.0 * x1 + 3.0 * x2 - 2.0 * t1 - t2;
    let c3 = 2.0 * x1 - 2.0 * x2 + t1 + t2;
    c0 + c1 * t + c2 * t * t + c3 * t * t * t
}

impl Curve {
    fn new(pts: Vec<V3>) -> Self {
        let mut c = Curve { pts, lengths: vec![] };
        let divisions = 200;
        let mut last = c.point(0.0);
        let mut sum = 0.0;
        c.lengths.push(0.0);
        for p in 1..=divisions {
            let cur = c.point(p as f64 / divisions as f64);
            sum += len(sub(cur, last));
            c.lengths.push(sum);
            last = cur;
        }
        c
    }
    fn length(&self) -> f64 {
        *self.lengths.last().unwrap_or(&0.0)
    }
    fn point(&self, t: f64) -> V3 {
        let p = &self.pts;
        let l = p.len();
        let pp = (l - 1) as f64 * t;
        let mut int = pp.floor() as usize;
        let mut weight = pp - int as f64;
        if weight == 0.0 && int == l - 1 {
            int = l - 2;
            weight = 1.0;
        }
        let p0 = if int > 0 { p[int - 1] } else { add(sub(p[0], p[1]), p[0]) };
        let p1 = p[int];
        let p2 = p[int + 1];
        let p3 = if int + 2 < l { p[int + 2] } else { add(sub(p[l - 1], p[l - 2]), p[l - 1]) };
        let d2 = |a: V3, b: V3| dot(sub(a, b), sub(a, b));
        let mut dt0 = d2(p0, p1).powf(0.25);
        let mut dt1 = d2(p1, p2).powf(0.25);
        let mut dt2 = d2(p2, p3).powf(0.25);
        if dt1 < 1e-4 {
            dt1 = 1.0;
        }
        if dt0 < 1e-4 {
            dt0 = dt1;
        }
        if dt2 < 1e-4 {
            dt2 = dt1;
        }
        [0, 1, 2].map(|k| cubic(p0[k], p1[k], p2[k], p3[k], dt0, dt1, dt2, weight))
    }
    fn u_to_t(&self, u: f64) -> f64 {
        let a = &self.lengths;
        let il = a.len();
        let target = u * a[il - 1];
        let (mut low, mut high): (i64, i64) = (0, il as i64 - 1);
        while low <= high {
            let i = low + (high - low) / 2;
            let cmp = a[i as usize] - target;
            if cmp < 0.0 {
                low = i + 1;
            } else if cmp > 0.0 {
                high = i - 1;
            } else {
                high = i;
                break;
            }
        }
        let i = high.max(0) as usize;
        if a[i] == target {
            return i as f64 / (il - 1) as f64;
        }
        let before = a[i];
        let after = a[(i + 1).min(il - 1)];
        let seg = after - before;
        let frac = if seg == 0.0 { 0.0 } else { (target - before) / seg };
        (i as f64 + frac) / (il - 1) as f64
    }
    fn point_at(&self, u: f64) -> V3 {
        self.point(self.u_to_t(u))
    }
    fn tangent_at(&self, u: f64) -> V3 {
        let t = self.u_to_t(u);
        let delta = 0.0001;
        let t1 = (t - delta).max(0.0);
        let t2 = (t + delta).min(1.0);
        norm(sub(self.point(t2), self.point(t1)))
    }
}

// ---------------------------------------------------------------- geometry (three.js builders)

#[derive(Default)]
struct Geo {
    pos: Vec<V3>,
    nor: Vec<V3>,
    idx: Vec<u32>,
}

fn lathe(points: &[[f64; 2]], segments: usize) -> Geo {
    let n = points.len();
    let mut init_normals: Vec<V3> = Vec::with_capacity(n);
    let mut prev = [0.0; 3];
    for j in 0..n {
        if j == n - 1 {
            init_normals.push(prev);
        } else {
            let dx = points[j + 1][0] - points[j][0];
            let dy = points[j + 1][1] - points[j][1];
            let normal = [dy, -dx, 0.0];
            if j == 0 {
                prev = normal;
                init_normals.push(norm(normal));
            } else {
                init_normals.push(norm(add(normal, prev)));
                prev = normal;
            }
        }
    }
    let mut g = Geo::default();
    for i in 0..=segments {
        let phi = i as f64 / segments as f64 * 2.0 * PI;
        let (s, c) = (phi.sin(), phi.cos());
        for j in 0..n {
            g.pos.push([points[j][0] * s, points[j][1], points[j][0] * c]);
            let nn = init_normals[j];
            g.nor.push([nn[0] * s, nn[1], nn[0] * c]);
        }
    }
    for i in 0..segments {
        for j in 0..n - 1 {
            let base = (j + i * n) as u32;
            let (a, b, c, d) = (base, base + n as u32, base + n as u32 + 1, base + 1);
            g.idx.extend([a, b, d, c, d, b]);
        }
    }
    g
}

fn torus(radius: f64, tube: f64, radial: usize, tubular: usize, arc: f64) -> Geo {
    let mut g = Geo::default();
    for j in 0..=radial {
        for i in 0..=tubular {
            let u = i as f64 / tubular as f64 * arc;
            let v = j as f64 / radial as f64 * PI * 2.0;
            let p = [(radius + tube * v.cos()) * u.cos(), (radius + tube * v.cos()) * u.sin(), tube * v.sin()];
            let center = [radius * u.cos(), radius * u.sin(), 0.0];
            g.pos.push(p);
            g.nor.push(norm(sub(p, center)));
        }
    }
    let t = tubular as u32 + 1;
    for j in 1..=radial as u32 {
        for i in 1..=tubular as u32 {
            let a = t * j + i - 1;
            let b = t * (j - 1) + i - 1;
            let c = t * (j - 1) + i;
            let d = t * j + i;
            g.idx.extend([a, b, d, b, c, d]);
        }
    }
    g
}

fn cylinder(r_top: f64, r_bottom: f64, height: f64, radial: usize) -> Geo {
    let mut g = Geo::default();
    let half = height / 2.0;
    let slope = (r_bottom - r_top) / height;
    let mut grid = vec![];
    for y in 0..=1 {
        let v = y as f64;
        let radius = v * (r_bottom - r_top) + r_top;
        let mut row = vec![];
        for x in 0..=radial {
            let theta = x as f64 / radial as f64 * 2.0 * PI;
            let (s, c) = (theta.sin(), theta.cos());
            row.push(g.pos.len() as u32);
            g.pos.push([radius * s, -v * height + half, radius * c]);
            g.nor.push(norm([s, slope, c]));
        }
        grid.push(row);
    }
    for x in 0..radial {
        let (a, b, c, d) = (grid[0][x], grid[1][x], grid[1][x + 1], grid[0][x + 1]);
        if r_top > 0.0 {
            g.idx.extend([a, b, d]);
        }
        if r_bottom > 0.0 {
            g.idx.extend([b, c, d]);
        }
    }
    for top in [true, false] {
        let radius = if top { r_top } else { r_bottom };
        let sign = if top { 1.0 } else { -1.0 };
        let center_start = g.pos.len() as u32;
        for _ in 1..=radial {
            g.pos.push([0.0, half * sign, 0.0]);
            g.nor.push([0.0, sign, 0.0]);
        }
        let center_end = g.pos.len() as u32;
        for x in 0..=radial {
            let theta = x as f64 / radial as f64 * 2.0 * PI;
            g.pos.push([radius * theta.sin(), half * sign, radius * theta.cos()]);
            g.nor.push([0.0, sign, 0.0]);
        }
        for x in 0..radial as u32 {
            let (c, i) = (center_start + x, center_end + x);
            if top {
                g.idx.extend([i, i + 1, c]);
            } else {
                g.idx.extend([i + 1, i, c]);
            }
        }
    }
    g
}

fn sphere(radius: f64, width_seg: usize, height_seg: usize) -> Geo {
    let mut g = Geo::default();
    let mut grid = vec![];
    for iy in 0..=height_seg {
        let v = iy as f64 / height_seg as f64;
        let mut row = vec![];
        for ix in 0..=width_seg {
            let u = ix as f64 / width_seg as f64;
            let p = [
                -radius * (u * 2.0 * PI).cos() * (v * PI).sin(),
                radius * (v * PI).cos(),
                radius * (u * 2.0 * PI).sin() * (v * PI).sin(),
            ];
            row.push(g.pos.len() as u32);
            g.pos.push(p);
            g.nor.push(norm(p));
        }
        grid.push(row);
    }
    for iy in 0..height_seg {
        for ix in 0..width_seg {
            let (a, b, c, d) = (grid[iy][ix + 1], grid[iy][ix], grid[iy + 1][ix], grid[iy + 1][ix + 1]);
            if iy != 0 {
                g.idx.extend([a, b, d]);
            }
            if iy != height_seg - 1 {
                g.idx.extend([b, c, d]);
            }
        }
    }
    g
}

/// Capsule along Y: a cylinder of `length` capped by hemispheres of `radius` (three.js CapsuleGeometry).
fn capsule(radius: f64, length: f64, cap_seg: usize, radial: usize) -> Geo {
    // Profile rings from the bottom pole to the top pole: (ring radius, y, normal y-component, normal radial factor)
    let mut rings: Vec<(f64, f64, f64, f64)> = vec![];
    for k in 0..=cap_seg {
        let phi = -PI / 2.0 + (k as f64 / cap_seg as f64) * (PI / 2.0);
        rings.push((radius * phi.cos(), -length / 2.0 + radius * phi.sin(), phi.sin(), phi.cos()));
    }
    for k in 0..=cap_seg {
        let phi = (k as f64 / cap_seg as f64) * (PI / 2.0);
        rings.push((radius * phi.cos(), length / 2.0 + radius * phi.sin(), phi.sin(), phi.cos()));
    }
    let n = rings.len();
    let mut g = Geo::default();
    for i in 0..=radial {
        let theta = i as f64 / radial as f64 * 2.0 * PI;
        let (s, c) = (theta.sin(), theta.cos());
        for &(rr, y, ny, nr) in &rings {
            g.pos.push([rr * s, y, rr * c]);
            g.nor.push(norm([nr * s, ny, nr * c]));
        }
    }
    for i in 0..radial {
        for j in 0..n - 1 {
            let base = (j + i * n) as u32;
            let (a, b, c, d) = (base, base + n as u32, base + n as u32 + 1, base + 1);
            g.idx.extend([a, b, d, c, d, b]);
        }
    }
    g
}

fn body_geometry(r: f64, h: f64) -> Geo {
    // Waisted cylinder with rounded rims, revolved around Y.
    let half = h / 2.0;
    let steps = 10;
    let mut pts = vec![[0.0, -half]];
    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let y = -half + t * h;
        let waist = 1.0 - 0.12 * (PI * t).sin();
        let rim = (PI * t.min(1.0 - t) * 2.2 + 0.35).sin().min(1.0);
        pts.push([r * waist * (0.86 + 0.14 * rim), y]);
    }
    pts.push([0.0, half]);
    lathe(&pts, 28)
}

// ---------------------------------------------------------------- scene

struct Level {
    u: f64,
    r: f64,
    h: f64,
    spine: f64,
    tilt: f64,
    trans: f64,
}

fn spine_curve() -> Curve {
    // Sagittal S-curve, cervical on the left, sacrum on the right; +y points posterior.
    Curve::new(vec![
        [-5.1, 0.15, 0.0],
        [-4.1, -0.12, 0.0],
        [-2.6, 0.22, 0.0],
        [-0.6, 0.48, 0.0],
        [1.4, 0.32, 0.0],
        [2.9, -0.08, 0.0],
        [4.1, -0.12, 0.0],
        [4.9, 0.25, 0.0],
    ])
}

fn build_levels(curve: &Curve) -> Vec<Level> {
    let count = 24;
    let raw: Vec<(f64, f64, f64, f64, f64)> = (0..count)
        .map(|i| {
            let k = i as f64 / (count - 1) as f64;
            let region = if i < 7 { 'c' } else if i < 19 { 't' } else { 'l' };
            let spine = match region {
                'c' => 0.22 + k * 0.5,
                't' => 0.5,
                _ => 0.4,
            };
            let tilt = match region {
                'c' => 0.2,
                't' => 0.8,
                _ => 0.35,
            };
            let trans = match region {
                'c' => 0.16,
                't' => 0.26,
                _ => 0.36,
            };
            (0.2 + k * 0.2, 0.2 + k * 0.17, spine, tilt, trans)
        })
        .collect();
    // Space levels along the curve by their own size (+ a disc gap), so larger lumbar levels get more room.
    let l = curve.length();
    let steps: Vec<f64> = raw.iter().map(|v| v.1 * 1.32).collect();
    let total: f64 = steps.iter().sum();
    let scale = 0.84 * l / total;
    let mut acc = 0.03 * l;
    raw.iter()
        .zip(steps.iter())
        .map(|(v, s)| {
            let center = acc + s * scale / 2.0;
            acc += s * scale;
            Level { u: center / l, r: v.0, h: v.1, spine: v.2, tilt: v.3, trans: v.4 }
        })
        .collect()
}

#[derive(Default)]
struct Mesh {
    data: Vec<f32>, // interleaved position (3) + normal (3)
    idx: Vec<u32>,
}

impl Mesh {
    fn add(&mut self, g: &Geo, m: &M4) {
        let base = (self.data.len() / 6) as u32;
        for (p, n) in g.pos.iter().zip(g.nor.iter()) {
            let wp = m.point(*p);
            let wn = m.normal(*n);
            self.data.extend([wp[0] as f32, wp[1] as f32, wp[2] as f32, wn[0] as f32, wn[1] as f32, wn[2] as f32]);
        }
        self.idx.extend(g.idx.iter().map(|i| i + base));
    }
}

/// Bakes the static part of the scene (everything inside the `position=[0,-0.25,0]` group).
fn build_scene() -> (Mesh, Mesh) {
    let curve = spine_curve();
    let levels = build_levels(&curve);
    let x_axis = [1.0, 0.0, 0.0];
    let frame = |u: f64| {
        let p = curve.point_at(u);
        let q = quat_from_units(x_axis, curve.tangent_at(u));
        M4::translate(p[0], p[1], p[2]).mul(&M4::quat(q))
    };
    let offset = M4::translate(0.0, -0.25, 0.0);
    let mut bone = Mesh::default();
    let mut disc = Mesh::default();

    let placed: Vec<(M4, V3)> = levels.iter().map(|lv| (offset.mul(&frame(lv.u)), curve.point_at(lv.u))).collect();

    for (lv, (g, _)) in levels.iter().zip(placed.iter()) {
        let (r, h) = (lv.r, lv.h);
        let ring_r = r * 0.72;
        let tube = r * 0.2;
        let canal_y = r + ring_r * 0.75;
        // Body: lathe axis rotated from Y to X (along the spine)
        bone.add(&body_geometry(r, h), &g.mul(&M4::euler(0.0, 0.0, PI / 2.0)));
        // Neural arch around the spinal canal
        bone.add(&torus(ring_r, tube, 10, 20, PI), &g.mul(&M4::translate(0.0, canal_y, 0.0)).mul(&M4::euler(0.0, PI / 2.0, 0.0)));
        // Pedicles joining arch to body
        for s in [-1.0, 1.0] {
            bone.add(&cylinder(tube, tube * 1.1, r * 0.8, 10), &g.mul(&M4::translate(0.0, r * 0.9, s * ring_r)));
        }
        // Spinous process, tilted toward the sacrum
        let sp = g.mul(&M4::translate(0.0, canal_y + ring_r, 0.0)).mul(&M4::euler(0.0, 0.0, -lv.tilt)).mul(&M4::translate(0.0, lv.spine / 2.0, 0.0));
        bone.add(&capsule(tube * 0.95, lv.spine, 6, 10), &sp);
        // Transverse processes
        for s in [-1.0, 1.0] {
            let tp = g
                .mul(&M4::translate(0.0, canal_y * 0.85, s * ring_r))
                .mul(&M4::euler(s * (PI / 2.0 - 0.25), 0.0, 0.0))
                .mul(&M4::translate(0.0, lv.trans / 2.0, 0.0));
            bone.add(&capsule(tube * 0.75, lv.trans, 6, 10), &tp);
        }
    }

    // Intervertebral discs
    for i in 0..levels.len() - 1 {
        let (a, b) = (&levels[i], &levels[i + 1]);
        let u = (a.u + b.u) / 2.0;
        let r = (a.r + b.r) / 2.0;
        let gap = len(sub(placed[i].1, placed[i + 1].1)) - (a.h + b.h) / 2.0;
        let l = (gap * 0.92).max(0.03);
        disc.add(&cylinder(r * 0.95, r * 0.95, l, 24), &offset.mul(&frame(u)).mul(&M4::euler(0.0, 0.0, PI / 2.0)));
    }

    // Sacrum: five fused, tapering segments, then a short coccyx
    let last = levels.last().expect("levels");
    let start = last.u + 0.025;
    let segs = 8;
    for i in 0..segs {
        let u = (start + i as f64 * (0.985 - start) / (segs - 1) as f64).min(0.995);
        let g = offset.mul(&frame(u));
        let sacral = i < 5;
        let r = if sacral { 0.4 - i as f64 * 0.055 } else { 0.11 - (i as f64 - 5.0) * 0.025 };
        let l = if sacral { 0.34 } else { 0.12 };
        let wide = if sacral { 1.55 - i as f64 * 0.08 } else { 1.0 };
        bone.add(&cylinder(r, r * 1.04, l, 20), &g.mul(&M4::euler(0.0, 0.0, PI / 2.0)).mul(&M4::scale(1.0, 1.0, wide)));
        if i < 4 {
            bone.add(&sphere(r * 0.38, 12, 10), &g.mul(&M4::translate(0.0, r * 0.95, 0.0)).mul(&M4::scale(1.0, 0.6, 1.0)));
        }
    }
    (bone, disc)
}

// ---------------------------------------------------------------- shading (three.js MeshStandardMaterial)

const VERT: &str = r#"#version 300 es
precision highp float;
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
uniform mat4 modelViewMatrix;
uniform mat4 projectionMatrix;
out vec3 vViewPosition;
out vec3 vNormal;
void main() {
    vec4 mvPosition = modelViewMatrix * vec4(position, 1.0);
    vViewPosition = -mvPosition.xyz;
    vNormal = mat3(modelViewMatrix) * normal; // rigid transform: normal matrix == rotation
    gl_Position = projectionMatrix * mvPosition;
}
"#;

const FRAG: &str = r#"#version 300 es
precision highp float;
in vec3 vViewPosition;
in vec3 vNormal;
uniform vec3 diffuse;
uniform float roughness;
uniform float metalness;
uniform vec3 ambientLightColor;
uniform vec3 hemiSky;
uniform vec3 hemiGround;
uniform vec3 hemiDir;
uniform vec3 dirColor[2];
uniform vec3 dirDir[2];
uniform vec3 pointColor;
uniform vec3 pointPos;
out vec4 fragColor;
#define RECIPROCAL_PI 0.3183098861837907
#define EPSILON 1e-6
float pow2(float x) { return x * x; }
vec3 BRDF_Lambert(vec3 c) { return RECIPROCAL_PI * c; }
vec3 F_Schlick(vec3 f0, float f90, float dotVH) {
    float fresnel = exp2((-5.55473 * dotVH - 6.98316) * dotVH);
    return f0 * (1.0 - fresnel) + (f90 * fresnel);
}
float V_GGX_SmithCorrelated(float alpha, float dotNL, float dotNV) {
    float a2 = pow2(alpha);
    float gv = dotNL * sqrt(a2 + (1.0 - a2) * pow2(dotNV));
    float gl = dotNV * sqrt(a2 + (1.0 - a2) * pow2(dotNL));
    return 0.5 / max(gv + gl, EPSILON);
}
float D_GGX(float alpha, float dotNH) {
    float a2 = pow2(alpha);
    float denom = pow2(dotNH) * (a2 - 1.0) + 1.0;
    return RECIPROCAL_PI * a2 / pow2(denom);
}
vec3 BRDF_GGX(vec3 L, vec3 V, vec3 N, vec3 f0, float f90, float rough) {
    float alpha = pow2(rough);
    vec3 H = normalize(L + V);
    float dotNL = clamp(dot(N, L), 0.0, 1.0);
    float dotNV = clamp(dot(N, V), 0.0, 1.0);
    float dotNH = clamp(dot(N, H), 0.0, 1.0);
    float dotVH = clamp(dot(V, H), 0.0, 1.0);
    vec3 F = F_Schlick(f0, f90, dotVH);
    return F * (V_GGX_SmithCorrelated(alpha, dotNL, dotNV) * D_GGX(alpha, dotNH));
}
vec3 RRTAndODTFit(vec3 v) {
    vec3 a = v * (v + 0.0245786) - 0.000090537;
    vec3 b = v * (0.983729 * v + 0.4329510) + 0.238081;
    return a / b;
}
vec3 ACESFilmicToneMapping(vec3 color) {
    const mat3 ACESInputMat = mat3(vec3(0.59719, 0.07600, 0.02840), vec3(0.35458, 0.90834, 0.13383), vec3(0.04823, 0.01566, 0.83777));
    const mat3 ACESOutputMat = mat3(vec3(1.60475, -0.10208, -0.00327), vec3(-0.53108, 1.10813, -0.07276), vec3(-0.07367, -0.00605, 1.07602));
    color *= 1.0 / 0.6;
    color = ACESInputMat * color;
    color = RRTAndODTFit(color);
    color = ACESOutputMat * color;
    return clamp(color, 0.0, 1.0);
}
vec3 sRGBTransferOETF(vec3 c) {
    return mix(pow(c, vec3(0.41666)) * 1.055 - vec3(0.055), c * 12.92, vec3(lessThanEqual(c, vec3(0.0031308))));
}
void main() {
    vec3 normal = normalize(vNormal);
    vec3 diffuseColor = diffuse * (1.0 - metalness);
    vec3 dxy = max(abs(dFdx(normal)), abs(dFdy(normal)));
    float geometryRoughness = max(max(dxy.x, dxy.y), dxy.z);
    float rough = min(max(roughness, 0.0525) + geometryRoughness, 1.0);
    vec3 specularColor = mix(vec3(0.04), diffuse, metalness);
    vec3 V = normalize(vViewPosition);
    vec3 geometryPosition = -vViewPosition;

    vec3 directDiffuse = vec3(0.0);
    vec3 directSpecular = vec3(0.0);
    for (int i = 0; i < 2; i++) {
        vec3 L = dirDir[i];
        vec3 irradiance = clamp(dot(normal, L), 0.0, 1.0) * dirColor[i];
        directSpecular += irradiance * BRDF_GGX(L, V, normal, specularColor, 1.0, rough);
        directDiffuse += irradiance * BRDF_Lambert(diffuseColor);
    }
    {
        vec3 lVector = pointPos - geometryPosition;
        vec3 L = normalize(lVector);
        float d = length(lVector);
        vec3 color = pointColor * (1.0 / max(pow(d, 2.0), 0.01));
        vec3 irradiance = clamp(dot(normal, L), 0.0, 1.0) * color;
        directSpecular += irradiance * BRDF_GGX(L, V, normal, specularColor, 1.0, rough);
        directDiffuse += irradiance * BRDF_Lambert(diffuseColor);
    }
    float hemiWeight = 0.5 * dot(normal, hemiDir) + 0.5;
    vec3 irradiance = ambientLightColor + mix(hemiGround, hemiSky, hemiWeight);
    vec3 indirectDiffuse = irradiance * BRDF_Lambert(diffuseColor);

    vec3 outgoing = directDiffuse + indirectDiffuse + directSpecular;
    fragColor = vec4(sRGBTransferOETF(ACESFilmicToneMapping(outgoing)), 1.0);
}
"#;

/// sRGB hex color -> linear working space (three.js ColorManagement), times `intensity`.
fn lin(hex: u32, intensity: f64) -> [f32; 3] {
    let c = |v: u32| {
        let s = v as f64 / 255.0;
        let l = if s < 0.04045 { s * 0.0773993808 } else { (s * 0.9478672986 + 0.0521327014).powf(2.4) };
        (l * intensity) as f32
    };
    [c((hex >> 16) & 255), c((hex >> 8) & 255), c(hex & 255)]
}

// ---------------------------------------------------------------- WebGL renderer (browser only)

#[cfg(feature = "hydrate")]
mod gl {
    use super::*;
    use wasm_bindgen::JsCast;
    use web_sys::{WebGl2RenderingContext as GL, WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject};

    pub struct Renderer {
        gl: GL,
        program: WebGlProgram,
        meshes: Vec<(WebGlVertexArrayObject, i32, [f32; 3], f32, f32)>,
        u_mv: Option<WebGlUniformLocation>,
        u_proj: Option<WebGlUniformLocation>,
        u_diffuse: Option<WebGlUniformLocation>,
        u_rough: Option<WebGlUniformLocation>,
        u_metal: Option<WebGlUniformLocation>,
        view: M4,
    }

    fn shader(gl: &GL, kind: u32, src: &str) -> Option<web_sys::WebGlShader> {
        let s = gl.create_shader(kind)?;
        gl.shader_source(&s, src);
        gl.compile_shader(&s);
        if gl.get_shader_parameter(&s, GL::COMPILE_STATUS).as_bool() != Some(true) {
            leptos::logging::error!("shader: {:?}", gl.get_shader_info_log(&s));
            return None;
        }
        Some(s)
    }

    fn look_at(eye: V3, target: V3, up: V3) -> M4 {
        let z = norm(sub(eye, target));
        let x = norm(cross(up, z));
        let y = cross(z, x);
        // inverse of the camera's world matrix (rotation transposed, translated)
        let mut m = M4::identity().0;
        m[0] = x[0];
        m[4] = x[1];
        m[8] = x[2];
        m[1] = y[0];
        m[5] = y[1];
        m[9] = y[2];
        m[2] = z[0];
        m[6] = z[1];
        m[10] = z[2];
        m[12] = -dot(x, eye);
        m[13] = -dot(y, eye);
        m[14] = -dot(z, eye);
        M4(m)
    }

    fn perspective(fov_deg: f64, aspect: f64, near: f64, far: f64) -> M4 {
        let top = near * (fov_deg.to_radians() / 2.0).tan();
        let height = 2.0 * top;
        let width = aspect * height;
        let left = -0.5 * width;
        let right = left + width;
        let bottom = top - height;
        let mut m = [0.0; 16];
        m[0] = 2.0 * near / (right - left);
        m[5] = 2.0 * near / (top - bottom);
        m[8] = (right + left) / (right - left);
        m[9] = (top + bottom) / (top - bottom);
        m[10] = -(far + near) / (far - near);
        m[11] = -1.0;
        m[14] = -2.0 * far * near / (far - near);
        M4(m)
    }

    impl Renderer {
        pub fn new(canvas: &web_sys::HtmlCanvasElement) -> Option<Self> {
            let attrs = js_sys::Object::new();
            for (k, v) in [("alpha", true), ("antialias", true), ("premultipliedAlpha", true), ("depth", true), ("stencil", false)] {
                let _ = js_sys::Reflect::set(&attrs, &k.into(), &v.into());
            }
            let _ = js_sys::Reflect::set(&attrs, &"powerPreference".into(), &"low-power".into());
            let gl: GL = canvas.get_context_with_context_options("webgl2", &attrs).ok()??.dyn_into().ok()?;

            let program = gl.create_program()?;
            gl.attach_shader(&program, &shader(&gl, GL::VERTEX_SHADER, VERT)?);
            gl.attach_shader(&program, &shader(&gl, GL::FRAGMENT_SHADER, FRAG)?);
            gl.link_program(&program);
            if gl.get_program_parameter(&program, GL::LINK_STATUS).as_bool() != Some(true) {
                leptos::logging::error!("program: {:?}", gl.get_program_info_log(&program));
                return None;
            }
            gl.use_program(Some(&program));

            let (bone, disc) = build_scene();
            let mut meshes = vec![];
            // BONE "#efe6d6" roughness .5 metalness .02 · DISC "#9fbfd6" roughness .35 metalness .05
            for (mesh, color, rough, metal) in [(&bone, 0xefe6d6, 0.5f32, 0.02f32), (&disc, 0x9fbfd6, 0.35, 0.05)] {
                let vao = gl.create_vertex_array()?;
                gl.bind_vertex_array(Some(&vao));
                let vb = gl.create_buffer()?;
                gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vb));
                gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &js_sys::Float32Array::from(mesh.data.as_slice()), GL::STATIC_DRAW);
                gl.enable_vertex_attrib_array(0);
                gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 24, 0);
                gl.enable_vertex_attrib_array(1);
                gl.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, 24, 12);
                let ib = gl.create_buffer()?;
                gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&ib));
                gl.buffer_data_with_array_buffer_view(GL::ELEMENT_ARRAY_BUFFER, &js_sys::Uint32Array::from(mesh.idx.as_slice()), GL::STATIC_DRAW);
                meshes.push((vao, mesh.idx.len() as i32, lin(color, 1.0), rough, metal));
            }
            gl.bind_vertex_array(None);

            // Camera: position [0, 0.45, 8.3] looking at the origin (react-three-fiber default).
            let view = look_at([0.0, 0.45, 8.3], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
            let u = |n: &str| gl.get_uniform_location(&program, n);
            let v3 = |loc: Option<WebGlUniformLocation>, v: [f32; 3]| gl.uniform3fv_with_f32_array(loc.as_ref(), &v);
            let dir = |p: V3| {
                // light.position - target.position, transformDirection(viewMatrix)
                let m = &view.0;
                let d = norm([m[0] * p[0] + m[4] * p[1] + m[8] * p[2], m[1] * p[0] + m[5] * p[1] + m[9] * p[2], m[2] * p[0] + m[6] * p[1] + m[10] * p[2]]);
                [d[0] as f32, d[1] as f32, d[2] as f32]
            };
            v3(u("ambientLightColor"), lin(0xfff3e2, 0.55));
            v3(u("hemiSky"), lin(0xfff6e8, 0.45));
            v3(u("hemiGround"), lin(0x1f5c8f, 0.45));
            v3(u("hemiDir"), dir([0.0, 1.0, 0.0]));
            let dc: Vec<f32> = [lin(0xfff1dc, 1.6), lin(0x8fbde0, 0.9)].concat();
            let dd: Vec<f32> = [dir([-4.0, 6.0, 6.0]), dir([5.0, -2.0, -6.0])].concat();
            gl.uniform3fv_with_f32_array(u("dirColor[0]").as_ref(), &dc);
            gl.uniform3fv_with_f32_array(u("dirDir[0]").as_ref(), &dd);
            v3(u("pointColor"), lin(0xffe2b8, 0.6));
            let pp = view.point([0.0, 3.0, 4.0]);
            v3(u("pointPos"), [pp[0] as f32, pp[1] as f32, pp[2] as f32]);

            gl.enable(GL::DEPTH_TEST);
            gl.depth_func(GL::LEQUAL);
            gl.enable(GL::CULL_FACE);
            gl.cull_face(GL::BACK);
            gl.front_face(GL::CCW);
            gl.clear_color(0.0, 0.0, 0.0, 0.0);

            Some(Renderer {
                u_mv: u("modelViewMatrix"),
                u_proj: u("projectionMatrix"),
                u_diffuse: u("diffuse"),
                u_rough: u("roughness"),
                u_metal: u("metalness"),
                gl,
                program,
                meshes,
                view,
            })
        }

        /// Draws one frame. `roll` and `float_y` are the two animated values from the original useFrame.
        pub fn draw(&self, canvas: &web_sys::HtmlCanvasElement, roll: f64, float_y: f64) {
            let gl = &self.gl;
            let dpr = web_sys::window().map(|w| w.device_pixel_ratio()).unwrap_or(1.0).clamp(1.0, 2.0);
            let (cw, ch) = (canvas.client_width().max(1) as f64, canvas.client_height().max(1) as f64);
            let (w, h) = ((cw * dpr).round() as u32, (ch * dpr).round() as u32);
            if canvas.width() != w || canvas.height() != h {
                canvas.set_width(w);
                canvas.set_height(h);
            }
            gl.viewport(0, 0, w as i32, h as i32);
            gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
            gl.use_program(Some(&self.program));

            // float group: rotation [0.18, -0.32, 0.04], position.y animated; roll group: rotation.x animated
            let world = M4::translate(0.0, float_y, 0.0).mul(&M4::euler(0.18, -0.32, 0.04)).mul(&M4::euler(roll, 0.0, 0.0));
            let mv = self.view.mul(&world);
            let proj = perspective(32.0, cw / ch, 0.1, 1000.0);
            gl.uniform_matrix4fv_with_f32_array(self.u_mv.as_ref(), false, &mv.to_f32());
            gl.uniform_matrix4fv_with_f32_array(self.u_proj.as_ref(), false, &proj.to_f32());
            for (vao, count, color, rough, metal) in &self.meshes {
                gl.uniform3fv_with_f32_array(self.u_diffuse.as_ref(), color);
                gl.uniform1f(self.u_rough.as_ref(), *rough);
                gl.uniform1f(self.u_metal.as_ref(), *metal);
                gl.bind_vertex_array(Some(vao));
                gl.draw_elements_with_i32(GL::TRIANGLES, *count, GL::UNSIGNED_INT, 0);
            }
            gl.bind_vertex_array(None);
        }
    }
}

// ---------------------------------------------------------------- component

/// The rotating spine. Renders a soft placeholder until WebGL is ready; pauses when off-screen
/// and stays still when the user prefers reduced motion.
#[component]
pub fn Spine3D(#[prop(optional, into)] class: String) -> impl IntoView {
    let wrap = NodeRef::<Div>::new();
    let canvas = NodeRef::<Canvas>::new();
    let ready = RwSignal::new(false);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        use std::cell::{Cell, RefCell};
        use std::rc::Rc;
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        let (Some(wrap_el), Some(canvas_el)) = (wrap.get(), canvas.get()) else { return };
        let canvas_el: web_sys::HtmlCanvasElement = canvas_el;
        let Some(renderer) = gl::Renderer::new(&canvas_el) else { return };
        let reduced = super::browser::prefers_reduced_motion();

        struct State {
            roll: f64,
            elapsed: f64,
            last: Option<f64>,
        }
        // rotation={[0.35, 0, 0]} is the roll group's initial value
        let state = Rc::new(RefCell::new(State { roll: 0.35, elapsed: 0.0, last: None }));
        let renderer = Rc::new(renderer);
        let draw_now = {
            let (renderer, state, canvas_el) = (renderer.clone(), state.clone(), canvas_el.clone());
            move || {
                let s = state.borrow();
                renderer.draw(&canvas_el, s.roll, (s.elapsed * 0.8).sin() * 0.06);
            }
        };
        draw_now();
        ready.set(true);

        // Animation loop (requestAnimationFrame), running only while visible.
        let running = Rc::new(Cell::new(false));
        let raf: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
        // Id of the pending animation frame, so it can be cancelled before the closure is dropped
        // (otherwise the browser would call a freed closure after navigating away).
        let raf_id: Rc<Cell<Option<i32>>> = Rc::new(Cell::new(None));
        let request = {
            let (raf, raf_id) = (raf.clone(), raf_id.clone());
            move || {
                if let Some(cb) = raf.borrow().as_ref() {
                    raf_id.set(super::browser::window().request_animation_frame(cb.as_ref().unchecked_ref()).ok());
                }
            }
        };
        let cancel = {
            let raf_id = raf_id.clone();
            move || {
                if let Some(id) = raf_id.take() {
                    let _ = super::browser::window().cancel_animation_frame(id);
                }
            }
        };
        let tick = {
            let (state, running, draw_now, raf_id) = (state.clone(), running.clone(), draw_now.clone(), raf_id.clone());
            let request = request.clone();
            move |ts: f64| {
                raf_id.set(None);
                if !running.get() {
                    return;
                }
                {
                    let mut s = state.borrow_mut();
                    // Clamp the step so a long pause (tab hidden / scrolled away) does not jump the model.
                    let delta = s.last.map(|l| ((ts - l) / 1000.0).clamp(0.0, 0.1)).unwrap_or(0.0);
                    s.last = Some(ts);
                    s.roll += delta * 0.32;
                    s.elapsed += delta;
                }
                draw_now();
                request();
            }
        };
        *raf.borrow_mut() = Some(Closure::<dyn FnMut(f64)>::new(tick));
        let start = {
            let (running, state, cancel, request) = (running.clone(), state.clone(), cancel.clone(), request.clone());
            move || {
                if running.get() || reduced {
                    return;
                }
                running.set(true);
                state.borrow_mut().last = None;
                cancel(); // never run two frame chains at once
                request();
            }
        };
        let stop = {
            let running = running.clone();
            move || running.set(false)
        };

        let target: web_sys::Element = wrap_el.into();
        let io = super::browser::observe(&target, 0.05, "0px", move |visible, _| if visible { start() } else { stop() });
        // Redraw on resize while paused, so the canvas never shows a stretched frame.
        let off_resize = super::browser::on_window("resize", {
            let (running, draw_now) = (running.clone(), draw_now.clone());
            move || {
                if !running.get() {
                    draw_now();
                }
            }
        });
        super::browser::cleanup(move || {
            running.set(false);
            io.disconnect();
            off_resize();
            cancel();
            raf.borrow_mut().take();
        });
    });
    #[cfg(not(feature = "hydrate"))]
    let _ = (&wrap, &canvas);

    view! {
        <div node_ref=wrap class=class aria-label="Rotating 3D model of the human spine" role="img">
            <div style="position:relative;width:100%;height:100%;overflow:hidden">
                <canvas node_ref=canvas style="display:block;width:100%;height:100%"></canvas>
                // Soft placeholder while the WebAssembly module and WebGL start (same as the original loading state).
                <Show when=move || !ready.get()>
                    <div class="absolute inset-0 h-full w-full animate-pulse-soft rounded-3xl bg-gradient-to-r from-transparent via-sky/40 to-transparent"></div>
                </Show>
            </div>
        </div>
    }
}

/// Hero block: glow + floor shadow + the spine.
#[component]
pub fn HeroSpine() -> impl IntoView {
    view! {
        <div class="relative mt-4 h-56 sm:h-64 lg:h-[270px]">
            // Soft glow and floor shadow so the model sits on the page instead of floating on a hole
            <div class="pointer-events-none absolute inset-x-10 top-1/2 h-32 -translate-y-1/2 rounded-full bg-gradient-to-r from-sky/60 via-gold-soft/70 to-sky/50 blur-3xl"></div>
            <div class="pointer-events-none absolute inset-x-20 bottom-3 h-6 rounded-[50%] bg-ink/10 blur-xl"></div>
            <Spine3D class="relative h-full w-full"/>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scene_builds() {
        let (bone, disc) = build_scene();
        assert!(bone.idx.len() > 10_000);
        assert_eq!(disc.idx.len() % 3, 0);
        assert!(bone.data.iter().all(|v| v.is_finite()));
        let c = spine_curve();
        assert!((c.length() - 10.6).abs() < 1.0);
    }
}
