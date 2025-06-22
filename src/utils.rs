pub fn div_mod(a: i32, b: i32) -> (i32, i32) {
    if b == 0 {
        panic!("Division by zero");
    }
    (a / b, a % b)
}
