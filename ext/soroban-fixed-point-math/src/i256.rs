use soroban_sdk::{Env, I256};

use crate::soroban_fixed_point::SorobanFixedPoint;

impl SorobanFixedPoint for I256 {
    fn fixed_mul_floor(&self, env: &Env, y: &I256, denominator: &I256) -> I256 {
        mul_div_floor(env, &self, y, denominator)
    }

    fn fixed_mul_ceil(&self, env: &Env, y: &I256, denominator: &I256) -> I256 {
        mul_div_ceil(env, &self, y, denominator)
    }

    fn fixed_div_floor(&self, env: &Env, y: &I256, denominator: &I256) -> I256 {
        mul_div_floor(env, &self, denominator, y)
    }

    fn fixed_div_ceil(&self, env: &Env, y: &I256, denominator: &I256) -> I256 {
        mul_div_ceil(env, &self, denominator, y)
    }
}

/// Performs floor(x * y / z)
pub(crate) fn mul_div_floor(env: &Env, x: &I256, y: &I256, z: &I256) -> I256 {
    let zero = I256::from_i32(env, 0);
    let r = x.mul(&y);
    if r < zero || (r > zero && z.clone() < zero) {
        // ceiling is taken by default for a negative result
        let remainder = r.rem_euclid(&z);
        let one = I256::from_i32(env, 1);
        r.div(&z).sub(if remainder > zero { &one } else { &zero })
    } else {
        // floor taken by default for a positive or zero result
        r.div(&z)
    }
}

/// Performs ceil(x * y / z)
pub(crate) fn mul_div_ceil(env: &Env, x: &I256, y: &I256, z: &I256) -> I256 {
    let zero = I256::from_i32(env, 0);
    let r = x.mul(&y);
    if r <= zero || (r > zero && z.clone() < zero) {
        // ceiling is taken by default for a negative or zero result
        r.div(&z)
    } else {
        // floor taken by default for a positive result
        let remainder = r.rem_euclid(&z);
        let one = I256::from_i32(env, 1);
        r.div(&z).add(if remainder > zero { &one } else { &zero })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /********** fixed_mul_floor **********/

    #[test]
    fn test_fixed_mul_floor_rounds_down() {
        let env = Env::default();
        let x: I256 = I256::from_i128(&env, 1_5391283);
        let y: I256 = I256::from_i128(&env, 314_1592653);
        let denominator: I256 = I256::from_i128(&env, 1_0000001);

        let result = x.fixed_mul_floor(&env, &y, &denominator);

        assert_eq!(result, I256::from_i128(&env, 483_5313675));
    }

    #[test]
    fn test_fixed_mul_floor_negative_rounds_down() {
        let env = Env::default();
        let x: I256 = I256::from_i128(&env, -1_5391283);
        let y: I256 = I256::from_i128(&env, 314_1592653);
        let denominator: I256 = I256::from_i128(&env, 1_0000001);

        let result = x.fixed_mul_floor(&env, &y, &denominator);

        assert_eq!(result, I256::from_i128(&env, -483_5313676));
    }

    #[test]
    fn test_fixed_mul_floor_large_number() {
        let env = Env::default();
        let x: I256 = I256::from_i128(&env, i128::MAX);
        let y: I256 = I256::from_i128(&env, 10i128.pow(38));
        let denominator: I256 = I256::from_i128(&env, 10i128.pow(18));

        let result = x.clone().fixed_mul_floor(&env, &y, &denominator);

        let expected_result = x.mul(&I256::from_i128(&env, 10i128.pow(20)));
        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn test_fixed_mul_floor_phantom_overflow() {
        let env = Env::default();
        let x: I256 = I256::from_i128(&env, i128::MAX);
        // 256 bit max ~= 5.8e76, 128 bit max ~= 1.7e38, need to multiply by at least 10^39
        let y: I256 = I256::from_i128(&env, 10i128.pow(39));
        let denominator: I256 = I256::from_i128(&env, 10i128.pow(18));

        x.fixed_mul_floor(&env, &y, &denominator);
    }

    /********** fixed_mul_ceil **********/

    #[test]
    fn test_fixed_mul_ceil_rounds_up() {
        let env = Env::default();
        let x: I256 = I256::from_i128(&env, 1_5391283);
        let y: I256 = I256::from_i128(&env, 314_1592653);
        let denominator: I256 = I256::from_i128(&env, 1_0000001);

        let result = x.fixed_mul_ceil(&env, &y, &denominator);

        assert_eq!(result, I256::from_i128(&env, 483_5313676));
    }

    #[test]
    fn test_fixed_mul_ceil_negative_rounds_up() {
        let env = Env::default();
        let x: I256 = I256::from_i128(&env, -1_5391283);
        let y: I256 = I256::from_i128(&env, 314_1592653);
        let denominator: I256 = I256::from_i128(&env, 1_0000001);

        let result = x.fixed_mul_ceil(&env, &y, &denominator);

        assert_eq!(result, I256::from_i128(&env, -483_5313675));
    }

    #[test]
    fn test_fixed_mul_ceil_large_number() {
        let env = Env::default();
        let x: I256 = I256::from_i128(&env, i128::MAX);
        let y: I256 = I256::from_i128(&env, 10i128.pow(38));
        let denominator: I256 = I256::from_i128(&env, 10i128.pow(18));

        let result = x.clone().fixed_mul_ceil(&env, &y, &denominator);

        let expected_result = x.mul(&I256::from_i128(&env, 10i128.pow(20)));
        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn test_fixed_mul_ceil_phantom_overflow() {
        let env = Env::default();
        let x: I256 = I256::from_i128(&env, i128::MAX);
        // 256 bit max ~= 5.8e76, 128 bit max ~= 1.7e38, need to multiply by at least 10^39
        let y: I256 = I256::from_i128(&env, 10i128.pow(39));
        let denominator: I256 = I256::from_i128(&env, 10i128.pow(18));

        x.fixed_mul_ceil(&env, &y, &denominator);
    }

    /********** fixed_div_floor **********/

    #[test]
    fn test_fixed_div_floor_rounds_down() {
        let env = Env::default();
        let x: I256 = I256::from_i128(&env, 314_1592653);
        let y: I256 = I256::from_i128(&env, 1_5391280);
        let denominator: I256 = I256::from_i128(&env, 1_0000000);

        let result = x.fixed_div_floor(&env, &y, &denominator);

        assert_eq!(result, I256::from_i128(&env, 204_1150997));
    }

    #[test]
    fn test_fixed_div_floor_negative_rounds_down() {
        let env = Env::default();
        let x: I256 = I256::from_i128(&env, 314_1592653);
        let y: I256 = I256::from_i128(&env, -1_5391280);
        let denominator: I256 = I256::from_i128(&env, 1_0000000);

        let result = x.fixed_div_floor(&env, &y, &denominator);

        assert_eq!(result, I256::from_i128(&env, -204_1150998));
    }

    #[test]
    fn test_fixed_div_floor_large_number() {
        let env = Env::default();
        let x: I256 = I256::from_i128(&env, i128::MAX);
        let y: I256 = I256::from_i128(&env, 10i128.pow(27));
        let denominator: I256 = I256::from_i128(&env, 10i128.pow(38));

        let result = x.clone().fixed_div_floor(&env, &y, &denominator);

        let expected_result = x.mul(&I256::from_i128(&env, 10i128.pow(11)));
        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn test_fixed_div_floor_phantom_overflow() {
        let env = Env::default();
        let x: I256 = I256::from_i128(&env, i128::MAX);
        let y: I256 = I256::from_i128(&env, 10i128.pow(27));
        // 256 bit max ~= 5.8e76, 128 bit max ~= 1.7e38, need to multiply by at least 10^39
        let denominator: I256 = I256::from_i128(&env, 10i128.pow(39));

        x.fixed_div_floor(&env, &y, &denominator);
    }

    /********** fixed_div_ceil **********/

    #[test]
    fn test_fixed_div_ceil_rounds_down() {
        let env = Env::default();
        let x: I256 = I256::from_i128(&env, 314_1592653);
        let y: I256 = I256::from_i128(&env, 1_5391280);
        let denominator: I256 = I256::from_i128(&env, 1_0000000);

        let result = x.fixed_div_ceil(&env, &y, &denominator);

        assert_eq!(result, I256::from_i128(&env, 204_1150998));
    }

    #[test]
    fn test_fixed_div_ceil_negative_rounds_down() {
        let env = Env::default();
        let x: I256 = I256::from_i128(&env, 314_1592653);
        let y: I256 = I256::from_i128(&env, -1_5391280);
        let denominator: I256 = I256::from_i128(&env, 1_0000000);

        let result = x.fixed_div_ceil(&env, &y, &denominator);

        assert_eq!(result, I256::from_i128(&env, -204_1150997));
    }

    #[test]
    fn test_fixed_div_ceil_large_number() {
        let env = Env::default();
        let x: I256 = I256::from_i128(&env, i128::MAX);
        let y: I256 = I256::from_i128(&env, 10i128.pow(27));
        let denominator: I256 = I256::from_i128(&env, 10i128.pow(38));

        let result = x.clone().fixed_div_ceil(&env, &y, &denominator);

        let expected_result = x.mul(&I256::from_i128(&env, 10i128.pow(11)));
        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn test_fixed_div_ceil_phantom_overflow() {
        let env = Env::default();
        let x: I256 = I256::from_i128(&env, i128::MAX);
        let y: I256 = I256::from_i128(&env, 10i128.pow(27));
        // 256 bit max ~= 5.8e76, 128 bit max ~= 1.7e38, need to multiply by at least 10^39
        let denominator: I256 = I256::from_i128(&env, 10i128.pow(39));

        x.fixed_div_ceil(&env, &y, &denominator);
    }
}
