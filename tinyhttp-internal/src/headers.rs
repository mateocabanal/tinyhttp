#[derive(Default, Debug, Clone)]
pub struct HeaderMap {
    inner: Vec<(String, String)>,
}

impl HeaderMap {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner
            .iter()
            .rev()
            .find(|(name, _)| name.eq_ignore_ascii_case(key))
            .map(|(_, value)| value.as_str())
    }

    pub fn set(&mut self, key: impl AsRef<str>, val: impl AsRef<str>) {
        let key = key.as_ref().trim();
        let val = val.as_ref().trim();

        if let Some((_, value)) = self
            .inner
            .iter_mut()
            .find(|(name, _)| name.eq_ignore_ascii_case(key))
        {
            value.clear();
            value.push_str(val);
            return;
        }

        self.inner.push((key.to_string(), val.to_string()));
    }

    pub fn contains(&self, key: &str) -> bool {
        self.inner
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case(key))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
