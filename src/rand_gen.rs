use std::ops::{Deref, Index};
use lazy_static::*;
use std::sync::Mutex;
use bracket_lib::prelude::RandomNumberGenerator;
use rand::{RngCore, thread_rng};

lazy_static! {
    static ref RND: Mutex<Option<RandomNumberGenerator>> = Mutex::new(None);
}

pub fn init_random_with_random_seed() {
    *RND.lock().unwrap() = Some(RandomNumberGenerator::seeded(thread_rng().next_u64()));
}

pub fn init_random_with_seed(seed: u64) {
    *RND.lock().unwrap() = Some(RandomNumberGenerator::seeded(seed));
}

pub fn get_random_between<T>(a: T, b: T) -> T
    where T: rand::distributions::uniform::SampleUniform + PartialOrd {

    if let Some(rng) = &mut *RND.lock().unwrap() {
        rng.range(a, b)
    } else {
        panic!("Random number generator not initialized! Please call `init_random_with_random_seed()` or `init_random_with_seed(u64)` before calling this function!");
    }
}

pub fn get_random_from<T>(v: &[T]) -> &T {
    let index = get_random_between(0, v.len() - 1);
    &v[index]
}
