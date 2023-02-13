#[cfg(not(test))]
pub fn get_rng() -> impl rand::Rng {
    rand::thread_rng()
}

#[cfg(test)]
mod test_rng {
    use std::{cell::RefCell, rc::Rc};

    use rand::{rngs::StdRng, Error, Rng, RngCore, SeedableRng};

    const TEST_RNG_SEED: u64 = 0;

    thread_local! {
        static TEST_RNG_INNER: Rc<RefCell<StdRng>> =
            Rc::new(RefCell::new(StdRng::seed_from_u64(TEST_RNG_SEED)));
    }

    struct TestRng(Rc<RefCell<StdRng>>);

    impl RngCore for TestRng {
        fn next_u32(&mut self) -> u32 {
            self.0.borrow_mut().next_u32()
        }

        fn next_u64(&mut self) -> u64 {
            self.0.borrow_mut().next_u64()
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            self.0.borrow_mut().fill_bytes(dest)
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
            self.0.borrow_mut().try_fill_bytes(dest)
        }
    }

    pub fn reset_rng() {
        TEST_RNG_INNER.with(|rng| *rng.borrow_mut() = StdRng::seed_from_u64(TEST_RNG_SEED));
    }

    pub fn get_rng() -> impl Rng {
        TestRng(TEST_RNG_INNER.with(|rng| Rc::clone(rng)))
    }
}

#[cfg(test)]
pub use test_rng::*;
