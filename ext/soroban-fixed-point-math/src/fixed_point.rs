/// Fixed point trait for computing fixed point calculations with native rust types.
pub trait FixedPoint: Sized {
    /// Safely calculates floor(x * y / denominator). Returns None if a phantom overflow
    /// occurs or if the denominator is 0.
    fn fixed_mul_floor(self, y: Self, denominator: Self) -> Option<Self>;

    /// Safely calculates ceil(x * y / denominator). Returns None if a phantom overflow
    /// occurs or if the denominator is 0.
    fn fixed_mul_ceil(self, y: Self, denominator: Self) -> Option<Self>;

    /// Safely calculates floor(x * denominator / y). Returns None if a phantom overflow
    /// occurs or if the denominator is 0.
    fn fixed_div_floor(self, y: Self, denominator: Self) -> Option<Self>;

    /// Safely calculates ceil(x * denominator / y). Returns None if a phantom overflow
    /// occurs or if the denominator is 0.
    fn fixed_div_ceil(self, y: Self, denominator: Self) -> Option<Self>;
}
