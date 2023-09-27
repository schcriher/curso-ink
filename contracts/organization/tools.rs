/// Function that computes the approximate square root of a number (fast)
pub fn sqrt_fast(v: i64) -> i64 {
    // https://github.com/chmike/fpsqrt/blob/df099181030e95d663d89e87d4bf2d36534776a5/fpsqrt.c#L51
    assert!(v >= 0, "sqrt input should be non-negative");

    let mut b: u64 = 1 << 62;
    let mut q: u64 = 0;
    let mut r: u64 = v as u64;

    while b > r {
        b >>= 2;
    }
    while b > 0 {
        let t = q + b;
        q >>= 1;
        if r >= t {
            r -= t;
            q += b;
        }
        b >>= 2;
    }

    q as i64
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn get_sqrt_fast_test() {
        assert_eq!(sqrt_fast(1), 1); //      1
        assert_eq!(sqrt_fast(2), 1); //      1.41…
        assert_eq!(sqrt_fast(10), 3); //     3.16…
        assert_eq!(sqrt_fast(16), 4); //     4
        assert_eq!(sqrt_fast(100), 10); //  10
        assert_eq!(sqrt_fast(500), 22); //  22.36…
    }
}
