use crate::fixed_point::FixedPoint;

impl FixedPoint for i64 {
    fn fixed_mul_floor(self, y: i64, denominator: i64) -> Option<i64> {
        mul_div_floor(self, y, denominator)
    }

    fn fixed_mul_ceil(self, y: i64, denominator: i64) -> Option<i64> {
        mul_div_ceil(self, y, denominator)
    }

    fn fixed_div_floor(self, y: i64, denominator: i64) -> Option<i64> {
        mul_div_floor(self, denominator, y)
    }

    fn fixed_div_ceil(self, y: i64, denominator: i64) -> Option<i64> {
        mul_div_ceil(self, denominator, y)
    }
}

/// Performs floor(x * y / z)
fn mul_div_floor(x: i64, y: i64, z: i64) -> Option<i64> {
    return match x.checked_mul(y) {
        Some(r) => {
            if r < 0 || (r > 0 && z < 0) {
                // ceiling is taken by default for a negative result
                let remainder = r.checked_rem_euclid(z)?;
                (r / z).checked_sub(if remainder > 0 { 1 } else { 0 })
            } else {
                // floor taken by default for a positive or zero result
                r.checked_div(z)
            }
        }
        None => {
            let res_i128 = crate::i128::mul_div_floor(x as i128, y as i128, z as i128)?;
            if res_i128 > i64::MAX as i128 {
                return None;
            }
            Some(res_i128 as i64)
        }
    };
}

/// Performs ceil(x * y / z)
fn mul_div_ceil(x: i64, y: i64, z: i64) -> Option<i64> {
    return match x.checked_mul(y) {
        Some(r) => {
            if r <= 0 || (r > 0 && z < 0) {
                // ceiling is taken by default for a negative or zero result
                r.checked_div(z)
            } else {
                // floor taken by default for a positive result
                let remainder = r.checked_rem_euclid(z)?;
                (r / z).checked_add(if remainder > 0 { 1 } else { 0 })
            }
        }
        None => {
            let res_i128 = crate::i128::mul_div_ceil(x as i128, y as i128, z as i128)?;
            if res_i128 > i64::MAX as i128 {
                return None;
            }
            Some(res_i128 as i64)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    /********** fixed_mul_floor **********/

    #[test]
    fn test_fixed_mul_floor_rounds_down() {
        let x: i64 = 1_5391283;
        let y: i64 = 314_1592653;
        let denominator: i64 = 1_0000001;

        let result = x.fixed_mul_floor(y, denominator).unwrap();

        assert_eq!(result, 483_5313675)
    }

    #[test]
    fn test_fixed_mul_floor_large_number() {
        let x: i64 = 9_223_372_036;
        let y: i64 = 1_000_000_000;
        let denominator: i64 = 1_000_000_000;

        let result = x.fixed_mul_floor(y, denominator).unwrap();

        assert_eq!(result, 9_223_372_036)
    }

    #[test]
    fn test_fixed_mul_floor_phantom_overflow_uses_i128() {
        let x: i64 = 9_223_372_036;
        let y: i64 = 2_000_000_000;
        let denominator: i64 = 1_000_000_000;

        let result = x.fixed_mul_floor(y, denominator).unwrap();

        assert_eq!(result, 18_446_744_072);
    }

    #[test]
    fn test_fixed_mul_floor_result_overflow() {
        let x: i64 = 9_223_372_036_000_000_000;
        let y: i64 = 2_000_000_000;
        let denominator: i64 = 1_000_000_000;

        let result = x.fixed_mul_floor(y, denominator);

        assert_eq!(result, None);
    }

    /********** fixed_mul_ceil **********/

    #[test]
    fn test_fixed_mul_ceil_rounds_up() {
        let x: i64 = 1_5391283;
        let y: i64 = 314_1592653;
        let denominator: i64 = 1_0000001;

        let result = x.fixed_mul_ceil(y, denominator).unwrap();

        assert_eq!(result, 483_5313676)
    }

    #[test]
    fn test_fixed_mul_ceil_large_number() {
        let x: i64 = 9_223_372_036;
        let y: i64 = 1_000_000_000;
        let denominator: i64 = 1_000_000_000;

        let result = x.fixed_mul_ceil(y, denominator).unwrap();

        assert_eq!(result, 9_223_372_036)
    }

    #[test]
    fn test_fixed_mul_ceil_phantom_overflow_uses_i128() {
        let x: i64 = 9_223_372_036;
        let y: i64 = 2_000_000_000;
        let denominator: i64 = 1_000_000_000;

        let result = x.fixed_mul_ceil(y, denominator).unwrap();

        assert_eq!(result, 18446744072);
    }

    #[test]
    fn test_fixed_mul_ceil_result_overflow() {
        let x: i64 = 9_223_372_036_000_000_000;
        let y: i64 = 2_000_000_000;
        let denominator: i64 = 1_000_000_000;

        let result = x.fixed_mul_ceil(y, denominator);

        assert_eq!(result, None);
    }

    /********** fixed_div_floor **********/

    #[test]
    fn test_fixed_div_floor_rounds_down() {
        let x: i64 = 314_1592653;
        let y: i64 = 1_5391280;
        let denominator: i64 = 1_0000000;

        let result = x.fixed_div_floor(y, denominator).unwrap();

        assert_eq!(result, 204_1150997)
    }

    #[test]
    fn test_fixed_div_floor_large_number() {
        let x: i64 = 9_223_372_036;
        let y: i64 = 1_000_000_000;
        let denominator: i64 = 1_000_000_000;

        let result = x.fixed_div_floor(y, denominator).unwrap();

        assert_eq!(result, 9_223_372_036)
    }

    #[test]
    fn test_fixed_div_floor_phantom_overflow_uses_i128() {
        let x: i64 = 9_223_372_036;
        let y: i64 = 1_000_000_000;
        let denominator: i64 = 2_000_000_000;

        let result = x.fixed_div_floor(y, denominator).unwrap();

        assert_eq!(result, 18_446_744_072);
    }

    #[test]
    fn test_fixed_div_floor_result_overflow() {
        let x: i64 = 9_223_372_036_000_000_000;
        let y: i64 = 1_000_000_000;
        let denominator: i64 = 2_000_000_000;

        let result = x.fixed_div_floor(y, denominator);

        assert_eq!(result, None);
    }

    /********** fixed_div_ceil **********/

    #[test]
    fn test_fixed_div_ceil_rounds_up() {
        let x: i64 = 314_1592653;
        let y: i64 = 1_5391280;
        let denominator: i64 = 1_0000000;

        let result = x.fixed_div_ceil(y, denominator).unwrap();

        assert_eq!(result, 204_1150998)
    }

    #[test]
    fn test_fixed_div_ceil_large_number() {
        let x: i64 = 9_223_372_036;
        let y: i64 = 1_000_000_000;
        let denominator: i64 = 1_000_000_000;

        let result = x.fixed_div_ceil(y, denominator).unwrap();

        assert_eq!(result, 9_223_372_036)
    }

    #[test]
    fn test_fixed_div_ceil_phantom_overflow_uses_i128() {
        let x: i64 = 9_223_372_036;
        let y: i64 = 1_000_000_000;
        let denominator: i64 = 2_000_000_000;

        let result = x.fixed_div_ceil(y, denominator).unwrap();

        assert_eq!(result, 18_446_744_072);
    }

    #[test]
    fn test_fixed_div_ceil_result_overflow() {
        let x: i64 = 9_223_372_036_000_000_000;
        let y: i64 = 1_000_000_000;
        let denominator: i64 = 2_000_000_000;

        let result = x.fixed_div_ceil(y, denominator);

        assert_eq!(result, None);
    }
}
