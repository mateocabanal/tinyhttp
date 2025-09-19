use std::collections::HashMap;

use unicase::Ascii;

#[derive(Default, Debug, Clone)]
pub struct HeaderMap {
    inner: HashMap<Ascii<String>, Ascii<String>>,
}

impl HeaderMap {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn get(&self, key: impl ToString) -> Option<&str> {
        self.inner
            .get(&Ascii::new(key.to_string()))
            .map(|u| u.as_str())
    }

    pub fn set(&mut self, key: impl ToString, val: impl ToString) {
        let _ = self
            .inner
            .insert(Ascii::new(key.to_string()), Ascii::new(val.to_string()));
    }

    pub fn contains(&self, key: impl ToString) -> bool {
        self.inner.contains_key(&Ascii::new(key.to_string()))
    }
}
