#[cfg(not(test))]
pub fn get_rng() -> impl rand::Rng {
    rand::thread_rng()
}

#[cfg(test)]
mod test_rng {
    use std::{cell::RefCell, rc::Rc};

    use delegate::delegate;
    use rand::{rngs::StdRng, Error, Rng, RngCore, SeedableRng};

    const TEST_RNG_SEED: u64 = 0;

    thread_local! {
        static TEST_RNG_INNER: Rc<RefCell<StdRng>> =
            Rc::new(RefCell::new(StdRng::seed_from_u64(TEST_RNG_SEED)));
    }

    struct TestRng(Rc<RefCell<StdRng>>);

    impl RngCore for TestRng {
        delegate! {
            to self.0.borrow_mut() {
                fn next_u32(&mut self) -> u32;
                fn next_u64(&mut self) -> u64;
                fn fill_bytes(&mut self, dest: &mut [u8]);
                fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error>;
            }
        }
    }

    pub fn reset_rng() {
        TEST_RNG_INNER.with(|rng| *rng.borrow_mut() = StdRng::seed_from_u64(TEST_RNG_SEED));
    }

    pub fn reseed_rng() {
        TEST_RNG_INNER.with(|rng| *rng.borrow_mut() = StdRng::from_entropy());
    }

    pub fn get_rng() -> impl Rng {
        TestRng(TEST_RNG_INNER.with(Rc::clone))
    }
}

#[cfg(test)]
pub use test_rng::*;
