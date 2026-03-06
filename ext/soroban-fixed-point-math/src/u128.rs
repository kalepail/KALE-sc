use soroban_sdk::{unwrap::UnwrapOptimized, Env, U256};

use crate::{fixed_point::FixedPoint, SorobanFixedPoint};

impl FixedPoint for u128 {
    fn fixed_mul_floor(self, y: u128, denominator: u128) -> Option<u128> {
        mul_div_floor(self, y, denominator)
    }

    fn fixed_mul_ceil(self, y: u128, denominator: u128) -> Option<u128> {
        mul_div_ceil(self, y, denominator)
    }

    fn fixed_div_floor(self, y: u128, denominator: u128) -> Option<u128> {
        mul_div_floor(self, denominator, y)
    }

    fn fixed_div_ceil(self, y: u128, denominator: u128) -> Option<u128> {
        mul_div_ceil(self, denominator, y)
    }
}

/// Performs floor(x * y / z)
pub(crate) fn mul_div_floor(x: u128, y: u128, z: u128) -> Option<u128> {
    let r = x.checked_mul(y)?;
    r.checked_div(z)
}

/// Performs ceil(x * y / z)
pub(crate) fn mul_div_ceil(x: u128, y: u128, z: u128) -> Option<u128> {
    let r = x.checked_mul(y)?;
    div_ceil(r, z)
}

/// Performs ceil(r / z)
fn div_ceil(r: u128, z: u128) -> Option<u128> {
    // floor taken by default for a positive result
    let remainder = r.checked_rem_euclid(z)?;
    (r / z).checked_add(if remainder > 0 { 1 } else { 0 })
}

impl SorobanFixedPoint for u128 {
    fn fixed_mul_floor(&self, env: &Env, y: &u128, denominator: &u128) -> u128 {
        scaled_mul_div_floor(self, env, y, denominator)
    }

    fn fixed_mul_ceil(&self, env: &Env, y: &u128, denominator: &u128) -> u128 {
        scaled_mul_div_ceil(&self, env, y, denominator)
    }

    fn fixed_div_floor(&self, env: &Env, y: &u128, denominator: &u128) -> u128 {
        scaled_mul_div_floor(self, env, denominator, y)
    }

    fn fixed_div_ceil(&self, env: &Env, y: &u128, denominator: &u128) -> u128 {
        scaled_mul_div_ceil(self, env, denominator, y)
    }
}

/// Performs floor(x * y / z)
fn scaled_mul_div_floor(x: &u128, env: &Env, y: &u128, z: &u128) -> u128 {
    return match x.checked_mul(*y) {
        Some(r) => r.checked_div(*z).unwrap_optimized(),
        None => {
            // scale to U256 and retry
            let res = crate::u256::mul_div_floor(
                &U256::from_u128(&env, *x),
                &U256::from_u128(&env, *y),
                &U256::from_u128(&env, *z),
            );
            // will panic if result is not representable in u128
            res.to_u128().unwrap_optimized()
        }
    };
}

/// Performs floor(x * y / z)
fn scaled_mul_div_ceil(x: &u128, env: &Env, y: &u128, z: &u128) -> u128 {
    return match x.checked_mul(*y) {
        Some(r) => div_ceil(r, *z).unwrap_optimized(),
        None => {
            // scale to U256 and retry
            let res = crate::u256::mul_div_ceil(
                &env,
                &U256::from_u128(&env, *x),
                &U256::from_u128(&env, *y),
                &U256::from_u128(&env, *z),
            );
            // will panic if result is not representable in u128
            res.to_u128().unwrap_optimized()
        }
    };
}

#[cfg(test)]
mod test_fixed_point {

    /********** fixed_mul_floor **********/

    use crate::FixedPoint;

    #[test]
    fn test_fixed_mul_floor_rounds_down() {
        let x: u128 = 1_5391283;
        let y: u128 = 314_1592653;
        let denominator: u128 = 1_0000001;

        let result = x.fixed_mul_floor(y, denominator).unwrap();

        assert_eq!(result, 483_5313675)
    }

    #[test]
    fn test_fixed_mul_floor_large_number() {
        let x: u128 = 340_282_366_920_938_463_463;
        let y: u128 = 1_000_000_000_000_000_000;
        let denominator: u128 = 1_000_000_000_000_000_000;

        let result = x.fixed_mul_floor(y, denominator).unwrap();

        assert_eq!(result, 340_282_366_920_938_463_463)
    }

    #[test]
    fn test_fixed_mul_floor_phantom_overflow() {
        let x: u128 = 340_282_366_920_938_463_463;
        let y: u128 = 1_000_000_000_000_000_001;
        let denominator: u128 = 1_000_000_000_000_000_000;

        let result = x.fixed_mul_floor(y, denominator);

        assert_eq!(None, result);
    }

    /********** fixed_mul_ceil **********/

    #[test]
    fn test_fixed_mul_ceil_rounds_up() {
        let x: u128 = 1_5391283;
        let y: u128 = 314_1592653;
        let denominator: u128 = 1_0000001;

        let result = x.fixed_mul_ceil(y, denominator).unwrap();

        assert_eq!(result, 483_5313676)
    }

    #[test]
    fn test_fixed_mul_ceil_large_number() {
        let x: u128 = 340_282_366_920_938_463_463;
        let y: u128 = 1_000_000_000_000_000_000;
        let denominator: u128 = 1_000_000_000_000_000_000;

        let result = x.fixed_mul_ceil(y, denominator).unwrap();

        assert_eq!(result, 340_282_366_920_938_463_463)
    }

    #[test]
    fn test_fixed_mul_ceil_phantom_overflow() {
        let x: u128 = 340_282_366_920_938_463_463;
        let y: u128 = 1_000_000_000_000_000_001;
        let denominator: u128 = 1_000_000_000_000_000_000;

        let result = x.fixed_mul_ceil(y, denominator);

        assert_eq!(None, result);
    }

    /********** fixed_div_floor **********/

    #[test]
    fn test_fixed_div_floor_rounds_down() {
        let x: u128 = 314_1592653;
        let y: u128 = 1_5391280;
        let denominator: u128 = 1_0000000;

        let result = x.fixed_div_floor(y, denominator).unwrap();

        assert_eq!(result, 204_1150997)
    }

    #[test]
    fn test_fixed_div_floor_large_number() {
        let x: u128 = 340_282_366_920_938_463_463;
        let y: u128 = 1_000_000_000_000_000_000;
        let denominator: u128 = 1_000_000_000_000_000_000;

        let result = x.fixed_div_floor(y, denominator).unwrap();

        assert_eq!(result, 340_282_366_920_938_463_463)
    }

    #[test]
    fn test_fixed_div_floor_phantom_overflow() {
        let x: u128 = 340_282_366_920_938_463_463;
        let y: u128 = 1_000_000_000_000_000_000;
        let denominator: u128 = 1_000_000_000_000_000_001;

        let result = x.fixed_div_floor(y, denominator);

        assert_eq!(None, result);
    }

    /********** fixed_div_ceil **********/

    #[test]
    fn test_fixed_div_ceil_rounds_down() {
        let x: u128 = 314_1592653;
        let y: u128 = 1_5391280;
        let denominator: u128 = 1_0000000;

        let result = x.fixed_div_ceil(y, denominator).unwrap();

        assert_eq!(result, 204_1150998)
    }

    #[test]
    fn test_fixed_div_ceil_large_number() {
        let x: u128 = 340_282_366_920_938_463_463;
        let y: u128 = 1_000_000_000_000_000_000;
        let denominator: u128 = 1_000_000_000_000_000_000;

        let result = x.fixed_div_ceil(y, denominator).unwrap();

        assert_eq!(result, 340_282_366_920_938_463_463)
    }

    #[test]
    fn test_fixed_div_ceil_phantom_overflow() {
        let x: u128 = 340_282_366_920_938_463_463;
        let y: u128 = 1_000_000_000_000_000_000;
        let denominator: u128 = 1_000_000_000_000_000_001;

        let result = x.fixed_div_ceil(y, denominator);

        assert_eq!(None, result);
    }
}

#[cfg(test)]
mod test_soroban_fixed_point {
    use crate::SorobanFixedPoint;
    use soroban_sdk::Env;

    /********** fixed_mul_floor **********/

    #[test]
    fn test_fixed_mul_floor_rounds_down() {
        let env = Env::default();
        let x: u128 = 1_5391283;
        let y: u128 = 314_1592653;
        let denominator: u128 = 1_0000001;

        let result = x.fixed_mul_floor(&env, &y, &denominator);

        assert_eq!(result, 483_5313675)
    }

    #[test]
    fn test_fixed_mul_floor_phantom_overflow_scales() {
        let env = Env::default();
        let x: u128 = 340_282_366_920_938_463_463;
        let y: u128 = 10u128.pow(27);
        let denominator: u128 = 10u128.pow(18);

        let result = x.fixed_mul_floor(&env, &y, &denominator);

        assert_eq!(result, 340_282_366_920_938_463_463 * 10u128.pow(9));
    }

    /********** fixed_mul_ceil **********/

    #[test]
    fn test_fixed_mul_ceil_rounds_up() {
        let env = Env::default();
        let x: u128 = 1_5391283;
        let y: u128 = 314_1592653;
        let denominator: u128 = 1_0000001;

        let result = x.fixed_mul_ceil(&env, &y, &denominator);

        assert_eq!(result, 483_5313676)
    }

    #[test]
    fn test_fixed_mul_ceil_large_number() {
        let env = Env::default();
        let x: u128 = 340_282_366_920_938_463_463;
        let y: u128 = 1_000_000_000_000_000_000;
        let denominator: u128 = 1_000_000_000_000_000_000;

        let result = x.fixed_mul_ceil(&env, &y, &denominator);

        assert_eq!(result, 340_282_366_920_938_463_463)
    }

    #[test]
    fn test_fixed_mul_ceil_phantom_overflow_scales() {
        let env = Env::default();
        let x: u128 = 340_282_366_920_938_463_463;
        let y: u128 = 10u128.pow(27);
        let denominator: u128 = 10u128.pow(18);

        let result = x.fixed_mul_ceil(&env, &y, &denominator);

        assert_eq!(result, 340_282_366_920_938_463_463 * 10u128.pow(9));
    }

    /********** fixed_div_floor **********/

    #[test]
    fn test_fixed_div_floor_rounds_down() {
        let env = Env::default();
        let x: u128 = 314_1592653;
        let y: u128 = 1_5391280;
        let denominator: u128 = 1_0000000;

        let result = x.fixed_div_floor(&env, &y, &denominator);

        assert_eq!(result, 204_1150997)
    }

    #[test]
    fn test_fixed_div_floor_phantom_overflow_scales() {
        let env = Env::default();
        let x: u128 = 340_282_366_920_938_463_463;
        let y: u128 = 10u128.pow(18);
        let denominator: u128 = 10u128.pow(27);

        let result = x.fixed_div_floor(&env, &y, &denominator);

        assert_eq!(result, 340_282_366_920_938_463_463 * 10u128.pow(9));
    }

    /********** fixed_div_ceil **********/

    #[test]
    fn test_fixed_div_ceil_rounds_down() {
        let env = Env::default();
        let x: u128 = 314_1592653;
        let y: u128 = 1_5391280;
        let denominator: u128 = 1_0000000;

        let result = x.fixed_div_ceil(&env, &y, &denominator);

        assert_eq!(result, 204_1150998)
    }

    #[test]
    fn test_fixed_div_ceil_large_number() {
        let env = Env::default();
        let x: u128 = 340_282_366_920_938_463_463;
        let y: u128 = 1_000_000_000_000_000_000;
        let denominator: u128 = 1_000_000_000_000_000_000;

        let result = x.fixed_div_ceil(&env, &y, &denominator);

        assert_eq!(result, 340_282_366_920_938_463_463)
    }

    #[test]
    fn test_fixed_div_ceil_phantom_overflow_scales() {
        let env = Env::default();
        let x: u128 = 340_282_366_920_938_463_463;
        let y: u128 = 10u128.pow(18);
        let denominator: u128 = 10u128.pow(27);

        let result = x.fixed_div_floor(&env, &y, &denominator);

        assert_eq!(result, 340_282_366_920_938_463_463 * 10u128.pow(9));
    }
}
