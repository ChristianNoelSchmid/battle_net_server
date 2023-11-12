use rand::{thread_rng, RngCore};


pub fn single(sides: u32) -> u32 {
    assert!(sides > 0);
    thread_rng().next_u32() % sides + 1
}

pub fn roll(sides: u32, count: u32) -> u32 {
    assert!(count > 0);
    (0..count).map(|_| single(sides)).sum()
}