use std::println;

use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::Env;
use std::time::{SystemTime, UNIX_EPOCH};
extern crate std;

// Example usage:
#[test]
fn main() {
    let env = Env::default();

    let zeros_min = 0;
    let zeros_max = 10;

    let gap_min = 0;
    let gap_max = 50;

    let stake_min = 0;
    let stake_max = 500_000_000_000;

    let block_reward = 25_000_000_000;

    let mut normalized_total = 0;

    for _i in 0..500 {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i128;
        let zeros = (timestamp % zeros_max + 1) as i128;
        let gap = (timestamp % (gap_max + 1)) as i128;
        let stake = (timestamp % (stake_max + 1)) as i128;
        let result = normalize(
            zeros, zeros_min, zeros_max, gap, gap_min, gap_max, stake, stake_min, stake_max,
        );
        normalized_total += result;
    }

    let zeros = 5;
    let gap = 10;
    let stake = 0;
    let result = normalize(
        zeros, zeros_min, zeros_max, gap, gap_min, gap_max, stake, stake_min, stake_max,
    );

    let reward = result.fixed_mul_floor(&env, &(block_reward), &normalized_total);

    println!("Total normalized rate: {}", normalized_total);
    println!("My normalized rate: {}", result);
    println!("{}", block_reward);
    println!("{}", reward);
}

/// Normalize three values given dynamic minimums and maximums using integer math.
///
/// Each input value is first clamped to its [min, max] range, then scaled to a fixed–point value
/// where the maximum maps to `scale` (here, 10,000 representing 100.00%).
///
/// # Parameters
/// - `zeros`: The submitted value for the "zeros" range.
/// - `zeros_min` and `zeros_max`: The current minimum and maximum for the "zeros" range.
/// - `gap`: The submitted value for the "gap" range.
/// - `gap_min` and `gap_max`: The current minimum and maximum for the "gap" range.
/// - `stake`: The submitted value for the "stake" range.
/// - `stake_min` and `stake_max`: The current minimum and maximum for the "stake" range.
///
/// # Returns
/// The unified normalized rate as an integer, where, for example, 7500 represents 75.00%.
fn normalize(
    zeros: i128,
    zeros_min: i128,
    zeros_max: i128,
    gap: i128,
    gap_min: i128,
    gap_max: i128,
    stake: i128,
    stake_min: i128,
    stake_max: i128,
) -> i128 {
    let env = Env::default();

    // Prevent division by zero by ensuring max > min for each range.
    if zeros_max <= zeros_min || gap_max <= gap_min || stake_max <= stake_min {
        return 0;
    }

    let scale = 10_000_000_000_000_000;

    // Clamp each value within its range.
    let clamped_zeros = zeros.max(zeros_min).min(zeros_max);
    let clamped_gap = gap.max(gap_min).min(gap_max);
    let clamped_stake = stake.max(stake_min).min(stake_max);

    // Normalize each value by subtracting the minimum and scaling relative to the range size.
    let norm_zeros =
        (clamped_zeros - zeros_min).fixed_mul_floor(&env, &scale, &(zeros_max - zeros_min));
    let norm_gap = (clamped_gap - gap_min).fixed_mul_floor(&env, &scale, &(gap_max - gap_min));
    let norm_stake =
        (clamped_stake - stake_min).fixed_mul_floor(&env, &scale, &(stake_max - stake_min));

    // Combine the three normalized values; here we simply take the average.
    norm_zeros + norm_gap + norm_stake
}
