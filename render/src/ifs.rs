use crate::params::FractalParams;
use crate::quaternion::*;
use rayon::prelude::*;

pub struct VoxelGrid {
    pub n: usize,
    pub data: Vec<[f32; 3]>,
}

impl VoxelGrid {
    pub fn new_sphere(n: usize) -> Self {
        let mut data = vec![[0.001f32; 3]; n * n * n];
        for iz in 0..n {
            let z = (iz as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            for iy in 0..n {
                let y = (iy as f32 + 0.5) / n as f32 * 2.0 - 1.0;
                for ix in 0..n {
                    let x = (ix as f32 + 0.5) / n as f32 * 2.0 - 1.0;
                    if x*x + y*y + z*z < 1.0 {
                        data[iz * n * n + iy * n + ix] = [1.0, 1.0, 1.0];
                    }
                }
            }
        }
        VoxelGrid { n, data }
    }

    pub fn new_empty(n: usize) -> Self {
        VoxelGrid { n, data: vec![[0.001f32; 3]; n * n * n] }
    }

    /// Trilinear interpolation, matching GL_LINEAR + CLAMP_TO_EDGE.
    pub fn sample(&self, u: f32, v: f32, w: f32) -> [f32; 3] {
        let n = self.n as f32;
        let fx = u * n - 0.5;
        let fy = v * n - 0.5;
        let fz = w * n - 0.5;

        let ix0 = fx.floor() as i32;
        let iy0 = fy.floor() as i32;
        let iz0 = fz.floor() as i32;

        let tx = fx - ix0 as f32;
        let ty = fy - iy0 as f32;
        let tz = fz - iz0 as f32;

        let clamp = |i: i32| -> usize { i.max(0).min(self.n as i32 - 1) as usize };

        let ix0 = clamp(ix0);
        let ix1 = clamp(ix0 as i32 + 1);
        let iy0 = clamp(iy0);
        let iy1 = clamp(iy0 as i32 + 1);
        let iz0 = clamp(iz0);
        let iz1 = clamp(iz0 as i32 + 1);

        let get = |ix: usize, iy: usize, iz: usize| -> [f32; 3] {
            self.data[iz * self.n * self.n + iy * self.n + ix]
        };

        let mut result = [0.0f32; 3];
        for c in 0..3 {
            let c000 = get(ix0, iy0, iz0)[c];
            let c100 = get(ix1, iy0, iz0)[c];
            let c010 = get(ix0, iy1, iz0)[c];
            let c110 = get(ix1, iy1, iz0)[c];
            let c001 = get(ix0, iy0, iz1)[c];
            let c101 = get(ix1, iy0, iz1)[c];
            let c011 = get(ix0, iy1, iz1)[c];
            let c111 = get(ix1, iy1, iz1)[c];

            let c00 = c000 * (1.0 - tx) + c100 * tx;
            let c10 = c010 * (1.0 - tx) + c110 * tx;
            let c01 = c001 * (1.0 - tx) + c101 * tx;
            let c11 = c011 * (1.0 - tx) + c111 * tx;

            let c0 = c00 * (1.0 - ty) + c10 * ty;
            let c1 = c01 * (1.0 - ty) + c11 * ty;

            result[c] = c0 * (1.0 - tz) + c1 * tz;
        }
        result
    }
}

fn color_weight(t: usize, cw: f32) -> [f32; 3] {
    match t {
        0 => [cw, 1.0, 1.0],
        1 => [1.0, cw, 1.0],
        2 => [1.0, 1.0, cw],
        3 => [cw, cw, 1.0],
        4 => [cw, 1.0, cw],
        _ => [1.0, cw, cw],
    }
}

/// Run one IFS iteration: read from `src`, write into `dst`.
fn iterate_once(src: &VoxelGrid, dst: &mut VoxelGrid, params: &FractalParams, scale: [f32; 3]) {
    let n = src.n;
    let ang = &params.ang;

    // Process z-slices in parallel
    dst.data.par_chunks_mut(n * n).enumerate().for_each(|(iz, slice)| {
        let wz = (iz as f32 + 0.5) / n as f32 * 2.0 - 1.0;
        for iy in 0..n {
            let wy = (iy as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            for ix in 0..n {
                let wx = (ix as f32 + 0.5) / n as f32 * 2.0 - 1.0;

                if wx*wx + wy*wy + wz*wz > 1.0 {
                    slice[iy * n + ix] = [0.001, 0.001, 0.001];
                    continue;
                }

                let wp: Q = [0.0, wx, wy, wz];
                let mut acc = [0.0f32; 3];
                let mut w_total = [0.0f32; 3];

                for t in 0..params.n_transforms {
                    let base = t * 24;

                    let num = eval_poly(wp, ang, base, params.num_deg, params.normalize);
                    let den = eval_poly(wp, ang, base + 12, params.den_deg, params.normalize);
                    let result = qmul(num, qinv(den));

                    let z = [result[1], result[2], result[3]];
                    let z2 = z[0]*z[0] + z[1]*z[1] + z[2]*z[2];
                    if z2 > 1.0 || z2.is_nan() { continue; }

                    let d2 = dot(den);
                    let j = (1.0 / (d2 * d2).max(1e-3)).min(50.0);

                    let s = [z[0] * 0.5 + 0.5, z[1] * 0.5 + 0.5, z[2] * 0.5 + 0.5];
                    let sampled = src.sample(s[0], s[1], s[2]);
                    let cwt = color_weight(t, params.color_weight);

                    for c in 0..3 {
                        acc[c] += sampled[c] * j * cwt[c];
                        w_total[c] += cwt[c];
                    }
                }

                let mut v = [0.0f32; 3];
                for c in 0..3 {
                    v[c] = if w_total[c] > 0.0 { acc[c] / w_total[c] } else { 0.0 };
                    v[c] = (v[c] * scale[c]).clamp(0.001, 100.0);
                }
                slice[iy * n + ix] = v;
            }
        }
    });
}

/// Compute adaptive scale from center region of the grid.
fn compute_scale(grid: &VoxelGrid, run_avg: &mut [f32; 3]) {
    let n = grid.n;
    let mid = n / 2;
    let r = (n / 16).max(2); // sample region radius
    let mut sums = [0.0f64; 3];
    let mut cnt = 0u64;
    for iz in (mid-r)..(mid+r) {
        for iy in (mid-r)..(mid+r) {
            for ix in (mid-r)..(mid+r) {
                let v = grid.data[iz * n * n + iy * n + ix];
                if v[0].is_finite() && v[1].is_finite() && v[2].is_finite() {
                    sums[0] += v[0] as f64;
                    sums[1] += v[1] as f64;
                    sums[2] += v[2] as f64;
                    cnt += 1;
                }
            }
        }
    }
    if cnt > 0 && (sums[0] + sums[1] + sums[2]) > 0.001 * cnt as f64 {
        for c in 0..3 {
            let avg = sums[c] / cnt as f64;
            let desired = if avg > 1e-10 { (0.5 / avg).min(10.0) } else { 1.0 };
            run_avg[c] = run_avg[c] * 0.95 + desired as f32 * 0.05;
        }
    }
}

/// Run the full IFS computation, returning the final grid.
pub fn compute_ifs(params: &FractalParams, n: usize, iterations: usize, pb: &indicatif::ProgressBar) -> VoxelGrid {
    let mut a = VoxelGrid::new_sphere(n);
    let mut b = VoxelGrid::new_empty(n);
    let mut run_avg = [1.0f32; 3];

    for i in 0..iterations {
        iterate_once(&a, &mut b, params, run_avg);
        compute_scale(&b, &mut run_avg);

        // Check for divergence
        let mid = n / 2;
        let v = b.data[mid * n * n + mid * n + mid];
        if !v[0].is_finite() || !v[1].is_finite() || !v[2].is_finite() {
            eprintln!("Warning: NaN detected at iteration {}, resetting", i);
            a = VoxelGrid::new_sphere(n);
            b = VoxelGrid::new_empty(n);
            run_avg = [1.0, 1.0, 1.0];
            continue;
        }

        std::mem::swap(&mut a, &mut b);
        pb.inc(1);
    }
    a
}
