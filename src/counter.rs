use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub struct Counter<K: Eq + Hash> {
    map: HashMap<K, u32>,
}

impl<K: Eq + Hash> Counter<K> {
    pub fn new() -> Counter<K> {
        Counter {
            map: HashMap::new(),
        }
    }

    pub fn increment(&mut self, key: K) {
        self.map.entry(key).and_modify(|e| *e += 1).or_insert(1);
    }

    pub fn get(&self, key: K) -> Option<&u32> {
        self.map.get(&key)
    }
}

impl From<Counter<String>> for Vec<(String, u32)> {
    fn from(counter: Counter<String>) -> Self {
        counter.map.into_iter().map(|(k, v)| (k, v)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_key_counted_once() {
        let mut counter = Counter::new();
        counter.increment("sidecar".to_string());
        assert_eq!(counter.get("sidecar".to_string()).unwrap(), &1);
    }

    #[test]
    fn existing_key_incremented_once() {
        let mut counter = Counter::new();
        counter.increment("sidecar".to_string());
        counter.increment("sidecar".to_string());
        assert_eq!(counter.get("sidecar".to_string()).unwrap(), &2);
    }

    #[test]
    fn from_counter_for_vec() {
        let mut counter = Counter::new();
        counter.increment("sidecar".to_string());
        let counts: Vec<(String, u32)> = counter.into();
        assert_eq!(counts[0].0, "sidecar".to_string());
        assert_eq!(counts[0].1, 1);
    }
}
