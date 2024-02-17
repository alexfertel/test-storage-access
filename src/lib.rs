#![feature(lazy_cell, const_trait_impl)]
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
#![cfg_attr(not(test), no_std)]
extern crate alloc;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

use alloc::vec::Vec;
use stylus_sdk::{alloy_primitives::U256, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct Counter {
        uint256 number;
    }
}

#[external]
impl Counter {
    pub fn number(&self) -> Result<U256, Vec<u8>> {
        Ok(self.number.get())
    }
}

#[cfg(test)]
mod tests {
    //! Stub `vm_hooks` needed for the tests to build.
    //!
    //! Functions here are behind the `test` feature, since normal contract execution should have the hosted
    //! stylus environment.
    use crate::Counter;
    use alloy_primitives::U256;
    use std::collections::HashMap;
    use std::ptr;
    use std::sync::LazyLock;
    use std::sync::Mutex;
    use stylus_sdk::storage::{StorageType, StorageU256};

    const WORD_BYTES: usize = 32;
    pub type Word = [u8; WORD_BYTES];

    pub static STORAGE: LazyLock<Mutex<HashMap<Word, Word>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));

    pub unsafe fn read_word(key: *const u8) -> Word {
        let mut res = Word::default();
        ptr::copy(key, res.as_mut_ptr(), WORD_BYTES);
        res
    }

    pub unsafe fn write_word(key: *mut u8, val: Word) {
        ptr::copy(val.as_ptr(), key, WORD_BYTES);
    }

    #[no_mangle]
    pub extern "C" fn storage_load_bytes32(key: *const u8, out: *mut u8) {
        let key = unsafe { read_word(key) };

        let value = STORAGE
            .lock()
            .unwrap()
            .get(&key)
            .map(Word::to_owned)
            .unwrap_or_default(); // defaults to zero value

        unsafe { write_word(out, value) };
    }

    #[test]
    fn gets_number() {
        let expected_n = U256::from(0);
        let counter = Counter {
            number: unsafe { StorageU256::new(U256::from(0), 0) },
        };

        let n = counter.number();
        assert_eq!(Ok(expected_n), n);
    }
}
