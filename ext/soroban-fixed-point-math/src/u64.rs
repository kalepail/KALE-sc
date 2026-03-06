use crate::fixed_point::FixedPoint;

impl FixedPoint for u64 {
    fn fixed_mul_floor(self, y: u64, denominator: u64) -> Option<u64> {
        mul_div_floor(self, y, denominator)
    }

    fn fixed_mul_ceil(self, y: u64, denominator: u64) -> Option<u64> {
        mul_div_ceil(self, y, denominator)
    }

    fn fixed_div_floor(self, y: u64, denominator: u64) -> Option<u64> {
        mul_div_floor(self, denominator, y)
    }

    fn fixed_div_ceil(self, y: u64, denominator: u64) -> Option<u64> {
        mul_div_ceil(self, denominator, y)
    }
}

/// Performs floor(x * y / z)
fn mul_div_floor(x: u64, y: u64, z: u64) -> Option<u64> {
    return match x.checked_mul(y) {
        Some(r) => r.checked_div(z),
        None => {
            let res_u128 = crate::u128::mul_div_floor(x as u128, y as u128, z as u128)?;
            if res_u128 > u64::MAX as u128 {
                return None;
            }
            Some(res_u128 as u64)
        }
    };
}

/// Performs ceil(x * y / z)
fn mul_div_ceil(x: u64, y: u64, z: u64) -> Option<u64> {
    return match x.checked_mul(y) {
        Some(r) => {
            let remainder = r.checked_rem_euclid(z)?;
            // div overflow will be caught by checked_rem_euclid
            (r / z).checked_add(if remainder > 0 { 1 } else { 0 })
        }
        None => {
            let res_u128 = crate::u128::mul_div_ceil(x as u128, y as u128, z as u128)?;
            if res_u128 > u64::MAX as u128 {
                return None;
            }
            Some(res_u128 as u64)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    /********** fixed_mul_floor **********/

    #[test]
    fn test_fixed_mul_floor_rounds_down() {
        let x: u64 = 1_5391283;
        let y: u64 = 314_1592653;
        let denominator: u64 = 1_0000001;

        let result = x.fixed_mul_floor(y, denominator).unwrap();

        assert_eq!(result, 483_5313675)
    }

    #[test]
    fn test_fixed_mul_floor_large_number() {
        let x: u64 = 18_446_744_073;
        let y: u64 = 1_000_000_000;
        let denominator: u64 = 1_000_000_000;

        let result = x.fixed_mul_floor(y, denominator).unwrap();

        assert_eq!(result, 18_446_744_073)
    }

    #[test]
    fn test_fixed_mul_floor_phantom_overflow_uses_u128() {
        let x: u64 = 18_446_744_073;
        let y: u64 = 2_000_000_000;
        let denominator: u64 = 1_000_000_000;

        let result = x.fixed_mul_floor(y, denominator).unwrap();

        assert_eq!(result, 36_893_488_146);
    }

    #[test]
    fn test_fixed_mul_floor_result_overflow() {
        let x: u64 = 18_446_744_073_000_000_000;
        let y: u64 = 2_000_000_000;
        let denominator: u64 = 1_000_000_000;

        let result = x.fixed_mul_floor(y, denominator);

        assert_eq!(result, None);
    }

    /********** fixed_mul_ceil **********/

    #[test]
    fn test_fixed_mul_ceil_rounds_up() {
        let x: u64 = 1_5391283;
        let y: u64 = 314_1592653;
        let denominator: u64 = 1_0000001;

        let result = x.fixed_mul_ceil(y, denominator).unwrap();

        assert_eq!(result, 483_5313676)
    }

    #[test]
    fn test_fixed_mul_ceil_large_number() {
        let x: u64 = 18_446_744_073;
        let y: u64 = 1_000_000_000;
        let denominator: u64 = 1_000_000_000;

        let result = x.fixed_mul_ceil(y, denominator).unwrap();

        assert_eq!(result, 18_446_744_073)
    }

    #[test]
    fn test_fixed_mul_ceil_phantom_overflow_uses_u128() {
        let x: u64 = 18_446_744_073;
        let y: u64 = 2_000_000_000;
        let denominator: u64 = 1_000_000_000;

        let result = x.fixed_mul_ceil(y, denominator).unwrap();

        assert_eq!(result, 36_893_488_146);
    }

    #[test]
    fn test_fixed_mul_ceil_result_overflow() {
        let x: u64 = 18_446_744_073_000_000_000;
        let y: u64 = 2_000_000_000;
        let denominator: u64 = 1_000_000_000;

        let result = x.fixed_mul_ceil(y, denominator);

        assert_eq!(result, None);
    }

    /********** fixed_div_floor **********/

    #[test]
    fn test_fixed_div_floor_rounds_down() {
        let x: u64 = 314_1592653;
        let y: u64 = 1_5391280;
        let denominator: u64 = 1_0000000;

        let result = x.fixed_div_floor(y, denominator).unwrap();

        assert_eq!(result, 204_1150997)
    }

    #[test]
    fn test_fixed_div_floor_large_number() {
        let x: u64 = 18_446_744_073;
        let y: u64 = 1_000_000_000;
        let denominator: u64 = 1_000_000_000;

        let result = x.fixed_div_floor(y, denominator).unwrap();

        assert_eq!(result, 18_446_744_073)
    }

    #[test]
    fn test_fixed_div_floor_phantom_overflow_uses_u128() {
        let x: u64 = 18_446_744_073;
        let y: u64 = 2_000_000_000;
        let denominator: u64 = 1_000_000_000;

        let result = x.fixed_div_floor(y, denominator).unwrap();

        assert_eq!(result, 9_223_372_036);
    }

    #[test]
    fn test_fixed_div_floor_result_overflow() {
        let x: u64 = 18_446_744_073_000_000_000;
        let y: u64 = 2_000_000_000;
        let denominator: u64 = 4_000_000_000;

        let result = x.fixed_div_floor(y, denominator);

        assert_eq!(result, None);
    }

    /********** fixed_div_ceil **********/

    #[test]
    fn test_fixed_div_ceil_rounds_up() {
        let x: u64 = 314_1592653;
        let y: u64 = 1_5391280;
        let denominator: u64 = 1_0000000;

        let result = x.fixed_div_ceil(y, denominator).unwrap();

        assert_eq!(result, 204_1150998)
    }

    #[test]
    fn test_fixed_div_ceil_large_number() {
        let x: u64 = 18_446_744_073;
        let y: u64 = 1_000_000_000;
        let denominator: u64 = 1_000_000_000;

        let result = x.fixed_div_ceil(y, denominator).unwrap();

        assert_eq!(result, 18_446_744_073)
    }

    #[test]
    fn test_fixed_div_ceil_phantom_overflow_uses_u128() {
        let x: u64 = 18_446_744_073;
        let y: u64 = 2_000_000_000;
        let denominator: u64 = 1_000_000_000;

        let result = x.fixed_div_ceil(y, denominator).unwrap();

        assert_eq!(result, 9_223_372_037);
    }

    #[test]
    fn test_fixed_div_ceil_result_overflow() {
        let x: u64 = 18_446_744_073_000_000_000;
        let y: u64 = 2_000_000_000;
        let denominator: u64 = 4_000_000_000;

        let result = x.fixed_div_ceil(y, denominator);

        assert_eq!(result, None);
    }
}
