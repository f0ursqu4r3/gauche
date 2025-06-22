pub fn div_mod(a: i32, b: i32) -> (i32, i32) {
    if b == 0 {
        panic!("Division by zero");
    }
    (a / b, a % b)
}

pub fn clamp(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub fn is_in_range(value: i32, min: i32, max: i32) -> bool {
    value >= min && value <= max
}
