//! The 2x3 affine transform PDF uses for both the graphics and text matrices.

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Matrix {
    pub(super) a: f64,
    pub(super) b: f64,
    pub(super) c: f64,
    pub(super) d: f64,
    pub(super) e: f64,
    pub(super) f: f64,
}

impl Matrix {
    pub(super) const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    pub(super) fn new(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Self {
        Self { a, b, c, d, e, f }
    }

    /// `self` applied first, then `other`.
    pub(super) fn then(self, other: Self) -> Self {
        Self {
            a: self.a * other.a + self.b * other.c,
            b: self.a * other.b + self.b * other.d,
            c: self.c * other.a + self.d * other.c,
            d: self.c * other.b + self.d * other.d,
            e: self.e * other.a + self.f * other.c + other.e,
            f: self.e * other.b + self.f * other.d + other.f,
        }
    }

    /// Horizontal scale factor, for turning a text-space advance into device units.
    pub(super) fn x_scale(self) -> f64 {
        self.a.hypot(self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_neutral() {
        let m = Matrix::new(2.0, 0.0, 0.0, 3.0, 4.0, 5.0);
        assert_eq!(m.then(Matrix::IDENTITY), m);
        assert_eq!(Matrix::IDENTITY.then(m), m);
    }

    #[test]
    fn translation_composes() {
        let a = Matrix::new(1.0, 0.0, 0.0, 1.0, 10.0, 20.0);
        let b = Matrix::new(1.0, 0.0, 0.0, 1.0, 5.0, 7.0);
        let m = a.then(b);
        assert_eq!((m.e, m.f), (15.0, 27.0));
    }

    #[test]
    fn a_flip_then_page_flip_restores_upright_text() {
        let text = Matrix::new(1.0, 0.0, 0.0, -1.0, 100.0, 348.0);
        let page_flip = Matrix::new(1.0, 0.0, 0.0, -1.0, 0.0, 792.0);
        let m = text.then(page_flip);
        assert_eq!(m.d, 1.0);
        assert_eq!(m.f, 444.0);
    }

    #[test]
    fn lower_text_lands_lower_after_a_page_flip() {
        let page_flip = Matrix::new(1.0, 0.0, 0.0, -1.0, 0.0, 792.0);
        let row = Matrix::new(1.0, 0.0, 0.0, -1.0, 0.0, 348.0).then(page_flip);
        let below = Matrix::new(1.0, 0.0, 0.0, -1.0, 0.0, 362.0).then(page_flip);
        assert!(below.f < row.f);
    }

    #[test]
    fn scaling_composes_into_x_scale() {
        let m = Matrix::new(3.0, 0.0, 0.0, 3.0, 0.0, 0.0)
            .then(Matrix::new(2.0, 0.0, 0.0, 2.0, 0.0, 0.0));
        assert_eq!(m.x_scale(), 6.0);
    }
}
