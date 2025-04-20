#[must_use]
pub const fn round_up(value: usize, scale: usize) -> usize {
    match value {
        0 => 0,
        size => (((size - 1) >> scale) + 1) << scale,
    }
}

#[must_use]
pub const fn round_down(value: usize, scale: usize) -> usize {
    (value >> scale) << scale
}
