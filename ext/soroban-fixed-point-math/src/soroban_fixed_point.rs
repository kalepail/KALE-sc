use soroban_sdk::Env;

// @dev - more detail about the forced panic can be found here: https://github.com/stellar/rs-soroban-env/pull/1091
//
/// Soroban fixed point trait for computing fixed point calculations with Soroban host objects.
///
/// Soroban host functions by default are non-recoverable. This means an arithmetic overflow or divide by zero will
/// result in a host panic instead of returning an error. For consistency, this trait will also panic in the same manner.
pub trait SorobanFixedPoint: Sized {
    /// Safely calculates floor(x * y / denominator).
    ///
    /// ### Panics
    /// This method will panic if the denominator is 0, a phantom overflow occurs, or
    /// the result does not fit in Self.
    fn fixed_mul_floor(&self, env: &Env, y: &Self, denominator: &Self) -> Self;

    /// Safely calculates ceil(x * y / denominator).
    ///
    /// ### Panics
    /// This method will panic if the denominator is 0, a phantom overflow occurs, or
    /// the result does not fit in Self.
    fn fixed_mul_ceil(&self, env: &Env, y: &Self, denominator: &Self) -> Self;

    /// Safely calculates floor(x * denominator / y).
    ///
    /// ### Panics
    /// This method will panic if the denominator is 0, a phantom overflow occurs, or
    /// the result does not fit in Self.
    fn fixed_div_floor(&self, env: &Env, y: &Self, denominator: &Self) -> Self;

    /// Safely calculates ceil(x * denominator / y).
    ///
    /// ### Panics
    /// This method will panic if the denominator is 0, a phantom overflow occurs, or
    /// the result does not fit in Self.
    fn fixed_div_ceil(&self, env: &Env, y: &Self, denominator: &Self) -> Self;
}
