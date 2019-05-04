use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

#[derive(Eq, PartialEq)]
pub struct TagCount<T: AsRef<str>> {
    tag: T,
    count: u32,
}

impl<T: AsRef<str>> TagCount<T> {
    pub fn new(tag: T, count: u32) -> Self {
        TagCount { tag, count }
    }
}

impl<T: Eq + Ord + AsRef<str>> Ord for TagCount<T> {
    fn cmp(&self, other: &TagCount<T>) -> Ordering {
        match other.count.cmp(&self.count) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.tag.cmp(&other.tag),
        }
    }
}

impl<T: Eq + Ord + AsRef<str>> PartialOrd for TagCount<T> {
    fn partial_cmp(&self, other: &TagCount<T>) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl<T: Eq + Ord + AsRef<str> + fmt::Display> fmt::Display for TagCount<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.tag, self.count)
    }
}

#[derive(Debug)]
pub struct Counter<'a> {
    map: HashMap<&'a str, u32>,
}

impl<'a> Counter<'a> {
    pub fn new() -> Counter<'a> {
        Counter {
            map: HashMap::new(),
        }
    }

    pub fn increment(&mut self, key: &'a str) {
        self.map.entry(key).and_modify(|e| *e += 1).or_insert(1);
    }

    pub fn get(&self, key: &'a str) -> Option<&u32> {
        self.map.get(key)
    }
}

impl<'a> From<Counter<'a>> for Vec<TagCount<&'a str>> {
    fn from(counter: Counter<'a>) -> Self {
        counter
            .map
            .into_iter()
            .map(|(k, v)| TagCount::new(k, v))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_key_counted_once() {
        let mut counter = Counter::new();
        counter.increment("sidecar");

        assert_eq!(counter.get("sidecar").unwrap(), &1);
    }

    #[test]
    fn existing_key_incremented_once() {
        let mut counter = Counter::new();
        counter.increment("sidecar");
        counter.increment("sidecar");

        assert_eq!(counter.get("sidecar").unwrap(), &2);
    }

    #[test]
    fn from_counter_for_vec() {
        let mut counter = Counter::new();
        counter.increment("sidecar");
        let counts: Vec<TagCount<&str>> = counter.into();

        assert_eq!(counts[0].tag, "sidecar");
        assert_eq!(counts[0].count, 1);
    }
}
