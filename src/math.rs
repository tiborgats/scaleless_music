/// Fast approximations.
pub trait MathFast<T> {
    /// fast sine
    fn sin_fast(self) -> T;
}

// Option:
// Fast sine with polynomial approximation, -π/2 to π/2
// Source: http://krisgarrett.net/papers/l2approx.pdf
// See also: http://www.ue.eti.pg.gda.pl/~wrona/lab_dsp/cw04/fun_aprox.pdf

impl MathFast<f32> for f32 {
    /// Tying out a fast sine approximation. Well, I found out that it is not faster
    /// than the `std::f32::sin()`...
    // http://lab.polygonal.de/2007/07/18/fast-and-accurate-sinecosine-approximation/
    fn sin_fast(self: f32) -> f32 {
        const A: f32 = 4.0 / ::std::f32::consts::PI;
        const B: f32 = 4.0 / (::std::f32::consts::PI * ::std::f32::consts::PI);
        let r = self % ::std::f32::consts::PI;
        if r < 0.0 {
            let s = (A + (B * r)) * r;
            0.225 * ((s * -s) - s) + s
        } else {
            let s = (A - (B * r)) * r;
            0.225 * ((s * s) - s) + s
        }
    }
}
