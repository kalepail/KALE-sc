#![no_std]

pub const STROOP: u64 = 1_0000000;

pub mod i128;
pub mod i256;
pub mod i64;
pub mod u128;
pub mod u256;
pub mod u64;

mod fixed_point;
pub use fixed_point::FixedPoint;
mod soroban_fixed_point;
pub use soroban_fixed_point::SorobanFixedPoint;
