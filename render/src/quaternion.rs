/// Quaternion as [real, i, j, k] — matches the shader's vec4(real, i, j, k).
pub type Q = [f32; 4];

pub fn qmul(p: Q, q: Q) -> Q {
    [
        p[0]*q[0] - p[1]*q[1] - p[2]*q[2] - p[3]*q[3],
        p[0]*q[1] + p[1]*q[0] + p[2]*q[3] - p[3]*q[2],
        p[0]*q[2] - p[1]*q[3] + p[2]*q[0] + p[3]*q[1],
        p[0]*q[3] + p[1]*q[2] - p[2]*q[1] + p[3]*q[0],
    ]
}

pub fn qinv(q: Q) -> Q {
    let d = (q[0]*q[0] + q[1]*q[1] + q[2]*q[2] + q[3]*q[3]).max(1e-10);
    [q[0]/d, -q[1]/d, -q[2]/d, -q[3]/d]
}

pub fn qnormalize(q: Q) -> Q {
    let n = (q[0]*q[0] + q[1]*q[1] + q[2]*q[2] + q[3]*q[3]).sqrt();
    if n > 1e-6 { [q[0]/n, q[1]/n, q[2]/n, q[3]/n] } else { q }
}

pub fn dot(q: Q) -> f32 {
    q[0]*q[0] + q[1]*q[1] + q[2]*q[2] + q[3]*q[3]
}

/// Read a quaternion from the coefficient array at the given index, optionally normalizing.
pub fn get_q(ang: &[f32], idx: usize, normalize: bool) -> Q {
    let q = [ang[idx], ang[idx+1], ang[idx+2], ang[idx+3]];
    if normalize { qnormalize(q) } else { q }
}

/// Horner evaluation of quaternion polynomial: c0*p^deg + c1*p^{deg-1} + ... + c_deg
/// Coefficients at offsets off, off+4, off+8.
pub fn eval_poly(p: Q, ang: &[f32], off: usize, deg: usize, normalize: bool) -> Q {
    let mut r = get_q(ang, off, normalize);
    for k in 1..=2 {
        if k > deg { break; }
        r = qadd(qmul(r, p), get_q(ang, off + k*4, normalize));
    }
    r
}

fn qadd(a: Q, b: Q) -> Q {
    [a[0]+b[0], a[1]+b[1], a[2]+b[2], a[3]+b[3]]
}
