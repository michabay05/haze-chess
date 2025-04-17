use crate::board::Board;

#[unsafe(no_mangle)]
pub fn my_add(a: i32, b: i32) -> i32 {
    return a + b + 102;
}

#[unsafe(no_mangle)]
pub fn board_size() -> usize {
    size_of::<Board>()
}
