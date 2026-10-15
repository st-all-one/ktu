//! Propriedades das portas (proptest): round-trip do `MemFs`.
//!
//! Propriedades puras sobre *fakes* determinísticos: não tocam no sistema de
//! ficheiros real.

use std::path::PathBuf;

use ktu_core::ports::{Fs, MemFs};
use proptest::collection::vec;
use proptest::prelude::any;
use proptest::{prop_assert, prop_assert_eq, proptest};

proptest! {
    #[test]
    fn mem_fs_round_trips_arbitrary_content(
        path in "[a-z/]{1,40}",
        content in vec(any::<u8>(), 0..256),
    ) {
        let fs = MemFs::new();
        let key = PathBuf::from(format!("/{path}"));
        prop_assert!(fs.write(&key, &content).is_ok());
        prop_assert_eq!(fs.read(&key).ok(), Some(content));
    }
}
