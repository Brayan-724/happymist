pub trait FloatExt {
    const SAFE_PRECISION: Self;

    /// Returns true if self is closer that [`Self::SAFE_PRECISION`] to rhs with a
    fn close_to(self, rhs: Self) -> bool;

    /// Safe division.
    /// Return 0 if rhs is less than [`Self::SAFE_PRECISION`]
    fn safe_div(self, rhs: Self) -> Self;
}

impl FloatExt for f32 {
    const SAFE_PRECISION: Self = 1.0e-6;

    fn close_to(self, rhs: Self) -> bool {
        (self - rhs).abs() < Self::SAFE_PRECISION
    }

    fn safe_div(self, rhs: Self) -> Self {
        if rhs.abs() < Self::SAFE_PRECISION {
            0f32
        } else {
            self / rhs
        }
    }
}
