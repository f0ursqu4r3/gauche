use glam::IVec2;

pub fn div_mod(a: i32, b: i32) -> (i32, i32) {
    if b == 0 {
        panic!("Division by zero");
    }
    (a / b, a % b)
}

pub fn new_york_dist(a: IVec2, b: IVec2) -> i32 {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();
    dx + dy
}
