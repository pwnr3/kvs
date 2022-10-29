use std::collections::HashMap;

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are stored in a `HashMap` in memory and not persisted to disk.
///
/// Example:
///
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::new();
/// store.set("key".to_owned(), "value".to_owned());
/// let val = store.get("key".to_owned());
/// assert_eq!(val, Some("value".to_owned()));
/// ```
#[derive(Default)]
pub struct KvStore {
    db: HashMap<String, String>,
}

impl KvStore {
    /// Initialize KvStore.
    pub fn new() -> Self {
        Self { db: HashMap::new() }
    }

    /// Set the value of a string key to a string.
    pub fn set(&mut self, key: String, val: String) {
        self.db.insert(key, val);
    }

    /// Get the string value of a given string key.
    ///
    /// Return `None` if the given key does not exist.
    pub fn get(&self, key: String) -> Option<String> {
        self.db.get(&key).map(String::from)
        /*
        https://users.rust-lang.org/t/convert-option-str-to-option-string/20533/2
        map(|s| => s.to_string())
        map(|s| => s.into())
        map(Into::into)
        map(ToOwned::to_owned)
        map(String::from)

        or
        .get(&key).clone()
        */
    }

    /// Remove a given key.
    pub fn remove(&mut self, key: String) {
        self.db.remove(&key);
    }
}
