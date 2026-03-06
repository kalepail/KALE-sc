use soroban_sdk::{Env, U256};

use crate::soroban_fixed_point::SorobanFixedPoint;

impl SorobanFixedPoint for U256 {
    fn fixed_mul_floor(&self, _env: &Env, y: &U256, denominator: &U256) -> U256 {
        mul_div_floor(self, y, denominator)
    }

    fn fixed_mul_ceil(&self, env: &Env, y: &U256, denominator: &U256) -> U256 {
        mul_div_ceil(env, self, y, denominator)
    }

    fn fixed_div_floor(&self, _env: &Env, y: &U256, denominator: &U256) -> U256 {
        mul_div_floor(self, denominator, y)
    }

    fn fixed_div_ceil(&self, env: &Env, y: &U256, denominator: &U256) -> U256 {
        mul_div_ceil(env, self, denominator, y)
    }
}

/// Performs floor(x * y / z)
pub(crate) fn mul_div_floor(x: &U256, y: &U256, z: &U256) -> U256 {
    // floor taken by default
    x.mul(&y).div(&z)
}

/// Performs ceil(x * y / z)
pub(crate) fn mul_div_ceil(env: &Env, x: &U256, y: &U256, z: &U256) -> U256 {
    let r = x.mul(&y);
    let remainder = r.rem_euclid(&z);
    let zero = U256::from_u32(env, 0);
    let one = U256::from_u32(env, 1);
    r.div(&z).add(if remainder > zero { &one } else { &zero })
}

#[cfg(test)]
mod tests {
    use super::*;

    /********** fixed_mul_floor **********/

    #[test]
    fn test_fixed_mul_floor_rounds_down() {
        let env = Env::default();
        let x: U256 = U256::from_u128(&env, 1_5391283);
        let y: U256 = U256::from_u128(&env, 314_1592653);
        let denominator: U256 = U256::from_u128(&env, 1_0000001);

        let result = x.fixed_mul_floor(&env, &y, &denominator);

        assert_eq!(result, U256::from_u128(&env, 483_5313675));
    }

    #[test]
    fn test_fixed_mul_floor_large_number() {
        let env = Env::default();
        let x: U256 = U256::from_u128(&env, u128::MAX);
        let y: U256 = U256::from_u128(&env, 10u128.pow(38));
        let denominator: U256 = U256::from_u128(&env, 10u128.pow(18));

        let result = x.clone().fixed_mul_floor(&env, &y, &denominator);

        let expected_result = x.mul(&U256::from_u128(&env, 10u128.pow(20)));
        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn test_fixed_mul_floor_phantom_overflow() {
        let env = Env::default();
        let x: U256 = U256::from_u128(&env, u128::MAX);
        // 256 bit max ~= 1.2e77, 128 bit max ~= 3.4e38, need to multiply by at least 10^39
        let y: U256 = U256::from_u128(&env, 10u128.pow(39));
        let denominator: U256 = U256::from_u128(&env, 10u128.pow(18));

        x.fixed_mul_floor(&env, &y, &denominator);
    }

    /********** fixed_mul_ceil **********/

    #[test]
    fn test_fixed_mul_ceil_rounds_up() {
        let env = Env::default();
        let x: U256 = U256::from_u128(&env, 1_5391283);
        let y: U256 = U256::from_u128(&env, 314_1592653);
        let denominator: U256 = U256::from_u128(&env, 1_0000001);

        let result = x.fixed_mul_ceil(&env, &y, &denominator);

        assert_eq!(result, U256::from_u128(&env, 483_5313676));
    }

    #[test]
    fn test_fixed_mul_ceil_large_number() {
        let env = Env::default();
        let x: U256 = U256::from_u128(&env, u128::MAX);
        let y: U256 = U256::from_u128(&env, 10u128.pow(38));
        let denominator: U256 = U256::from_u128(&env, 10u128.pow(18));

        let result = x.clone().fixed_mul_ceil(&env, &y, &denominator);

        let expected_result = x.mul(&U256::from_u128(&env, 10u128.pow(20)));
        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn test_fixed_mul_ceil_phantom_overflow() {
        let env = Env::default();
        let x: U256 = U256::from_u128(&env, u128::MAX);
        // 256 bit max ~= 1.2e77, 128 bit max ~= 3.4e38, need to multiply by at least 10^39
        let y: U256 = U256::from_u128(&env, 10u128.pow(39));
        let denominator: U256 = U256::from_u128(&env, 10u128.pow(18));

        x.fixed_mul_ceil(&env, &y, &denominator);
    }

    /********** fixed_div_floor **********/

    #[test]
    fn test_fixed_div_floor_rounds_down() {
        let env = Env::default();
        let x: U256 = U256::from_u128(&env, 314_1592653);
        let y: U256 = U256::from_u128(&env, 1_5391280);
        let denominator: U256 = U256::from_u128(&env, 1_0000000);

        let result = x.fixed_div_floor(&env, &y, &denominator);

        assert_eq!(result, U256::from_u128(&env, 204_1150997));
    }

    #[test]
    fn test_fixed_div_floor_large_number() {
        let env = Env::default();
        let x: U256 = U256::from_u128(&env, u128::MAX);
        let y: U256 = U256::from_u128(&env, 10u128.pow(27));
        let denominator: U256 = U256::from_u128(&env, 10u128.pow(38));

        let result = x.clone().fixed_div_floor(&env, &y, &denominator);

        let expected_result = x.mul(&U256::from_u128(&env, 10u128.pow(11)));
        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn test_fixed_div_floor_phantom_overflow() {
        let env = Env::default();
        let x: U256 = U256::from_u128(&env, u128::MAX);
        let y: U256 = U256::from_u128(&env, 10u128.pow(27));
        // 256 bit max ~= 1.2e77, 128 bit max ~= 3.4e38, need to multiply by at least 10^39
        let denominator: U256 = U256::from_u128(&env, 10u128.pow(39));

        x.fixed_div_floor(&env, &y, &denominator);
    }

    /********** fixed_div_ceil **********/

    #[test]
    fn test_fixed_div_ceil_rounds_down() {
        let env = Env::default();
        let x: U256 = U256::from_u128(&env, 314_1592653);
        let y: U256 = U256::from_u128(&env, 1_5391280);
        let denominator: U256 = U256::from_u128(&env, 1_0000000);

        let result = x.fixed_div_ceil(&env, &y, &denominator);

        assert_eq!(result, U256::from_u128(&env, 204_1150998));
    }

    #[test]
    fn test_fixed_div_ceil_large_number() {
        let env = Env::default();
        let x: U256 = U256::from_u128(&env, u128::MAX);
        let y: U256 = U256::from_u128(&env, 10u128.pow(27));
        let denominator: U256 = U256::from_u128(&env, 10u128.pow(38));

        let result = x.clone().fixed_div_ceil(&env, &y, &denominator);

        let expected_result = x.mul(&U256::from_u128(&env, 10u128.pow(11)));
        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn test_fixed_div_ceil_phantom_overflow() {
        let env = Env::default();
        let x: U256 = U256::from_u128(&env, u128::MAX);
        let y: U256 = U256::from_u128(&env, 10u128.pow(27));
        // 256 bit max ~= 1.2e77, 128 bit max ~= 3.4e38, need to multiply by at least 10^39
        let denominator: U256 = U256::from_u128(&env, 10u128.pow(39));

        x.fixed_div_ceil(&env, &y, &denominator);
    }
}
