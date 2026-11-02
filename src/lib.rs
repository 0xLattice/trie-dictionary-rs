//! Persistent compressed radix dictionary with Unicode grapheme edges.
use std::collections::BTreeMap;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;
#[derive(Clone, Debug, PartialEq, Eq)]
struct Node<V> {
    prefix: String,
    value: Option<V>,
    children: BTreeMap<String, Node<V>>,
}
#[derive(Clone, Debug)]
pub struct TrieDictionary<V> {
    root: Node<V>,
    len: usize,
}
impl<V> Default for TrieDictionary<V> {
    fn default() -> Self {
        Self {
            root: Node {
                prefix: String::new(),
                value: None,
                children: BTreeMap::new(),
            },
            len: 0,
        }
    }
}
fn first(s: &str) -> String {
    s.graphemes(true).next().unwrap_or("").nfc().collect()
}
fn common(a: &str, b: &str) -> (usize, usize) {
    a.graphemes(true)
        .zip(b.graphemes(true))
        .take_while(|(a, b)| a.nfc().eq(b.nfc()))
        .fold((0, 0), |(a, b), (x, y)| (a + x.len(), b + y.len()))
}
fn strip<'a>(key: &'a str, prefix: &str) -> Option<&'a str> {
    let (a, b) = common(prefix, key);
    if a == prefix.len() {
        Some(&key[b..])
    } else {
        None
    }
}
impl<V> TrieDictionary<V> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn get(&self, key: &str) -> Option<&V> {
        self.root.get(key)
    }
    pub fn insert(&mut self, key: impl Into<String>, value: V) -> Option<V> {
        let old = self.root.insert(&key.into(), value);
        if old.is_none() {
            self.len += 1
        }
        old
    }
    pub fn remove(&mut self, key: &str) -> Option<V> {
        let old = self.root.remove(key);
        if old.is_some() {
            self.len -= 1
        }
        old
    }
    pub fn clear(&mut self) {
        *self = Self::new()
    }
    pub fn iter(&self) -> Vec<(String, &V)> {
        let mut out = vec![];
        self.root.collect("", &mut out);
        out
    }
    pub fn keys(&self) -> Vec<String> {
        self.iter().into_iter().map(|(k, _)| k).collect()
    }
    pub fn values(&self) -> Vec<&V> {
        self.iter().into_iter().map(|(_, v)| v).collect()
    }
    pub fn values_along_path(&self, path: &str) -> Vec<&V> {
        if path.is_empty() || !self.root.children.contains_key(&first(path)) {
            return vec![];
        }
        let mut out: Vec<_> = self
            .iter()
            .into_iter()
            .filter(|(k, _)| !k.is_empty() && strip(path, k).is_some())
            .map(|(_, v)| v)
            .collect();
        if let Some(v) = &self.root.value {
            out.push(v)
        }
        out
    }
    pub fn child_characters(&self) -> Vec<&str> {
        self.root.children.keys().map(String::as_str).collect()
    }
    pub fn child_prefix(&self, c: &str) -> Option<&str> {
        self.root.children.get(&first(c)).map(|n| n.prefix.as_str())
    }
    pub fn is_fully_compressed(&self) -> bool {
        fn check<V>(n: &Node<V>) -> bool {
            (n.value.is_some() || n.children.len() != 1) && n.children.values().all(check)
        }
        self.root.children.values().all(check)
    }
    pub fn map_values<U>(&self, mut f: impl FnMut(&V) -> U) -> TrieDictionary<U> {
        self.iter().into_iter().map(|(k, v)| (k, f(v))).collect()
    }
    pub fn compact_map_values<U>(&self, mut f: impl FnMut(&V) -> Option<U>) -> TrieDictionary<U> {
        self.iter()
            .into_iter()
            .filter_map(|(k, v)| f(v).map(|v| (k, v)))
            .collect()
    }
}
impl<V: Clone> TrieDictionary<V> {
    pub fn setting(&self, key: impl Into<String>, value: V) -> Self {
        let mut next = self.clone();
        next.insert(key, value);
        next
    }
    pub fn removing(&self, key: &str) -> Self {
        let mut next = self.clone();
        next.remove(key);
        next
    }
    pub fn traverse(&self, prefix: &str) -> Self {
        self.iter()
            .into_iter()
            .filter_map(|(k, v)| strip(&k, prefix).map(|s| (s.to_string(), v.clone())))
            .collect()
    }
    pub fn traverse_child(&self, c: &str) -> Option<Self> {
        let child = self.root.children.get(&first(c))?.clone();
        let mut result = Self::new();
        result.root.value = self.root.value.clone();
        result.root.children.insert(c.into(), child);
        result.len = result.iter().len();
        Some(result)
    }
    pub fn traverse_to_next_child(&self, c: &str) -> Option<(String, Self)> {
        let prefix = self.child_prefix(c)?.to_owned();
        Some((prefix.clone(), self.traverse(&prefix)))
    }
    pub fn adding_prefix(&self, prefix: &str) -> Self {
        self.iter()
            .into_iter()
            .map(|(k, v)| (prefix.to_string() + &k, v.clone()))
            .collect()
    }
    pub fn filter(&self, mut f: impl FnMut(&str, &V) -> bool) -> Self {
        self.iter()
            .into_iter()
            .filter(|(k, v)| f(k, v))
            .map(|(k, v)| (k, v.clone()))
            .collect()
    }
    pub fn merge(&self, other: &Self, mut combine: impl FnMut(&V, &V) -> V) -> Self {
        let mut out = self.clone();
        for (k, v) in other.iter() {
            let v = out
                .get(&k)
                .map(|old| combine(old, v))
                .unwrap_or_else(|| v.clone());
            out.insert(k, v);
        }
        out
    }
    pub fn map_keys(&self, mut f: impl FnMut(&str) -> String) -> Self {
        self.iter()
            .into_iter()
            .map(|(k, v)| (f(&k), v.clone()))
            .collect()
    }
}
impl<V> FromIterator<(String, V)> for TrieDictionary<V> {
    fn from_iter<T: IntoIterator<Item = (String, V)>>(iter: T) -> Self {
        let mut t = Self::new();
        for (k, v) in iter {
            t.insert(k, v);
        }
        t
    }
}
impl<V> Node<V> {
    fn get(&self, key: &str) -> Option<&V> {
        let rest = strip(key, &self.prefix)?;
        if rest.is_empty() {
            self.value.as_ref()
        } else {
            self.children.get(&first(rest))?.get(rest)
        }
    }
    fn insert(&mut self, key: &str, value: V) -> Option<V> {
        let (shared, key_shared) = common(&self.prefix, key);
        if shared < self.prefix.len() {
            let old = Node {
                prefix: self.prefix[shared..].into(),
                value: self.value.take(),
                children: std::mem::take(&mut self.children),
            };
            let branch = first(&old.prefix).to_string();
            self.prefix.truncate(shared);
            self.children.insert(branch, old);
        }
        let rest = &key[key_shared..];
        if rest.is_empty() {
            return self.value.replace(value);
        }
        if let Some(child) = self.children.get_mut(&first(rest)) {
            child.insert(rest, value)
        } else {
            self.children.insert(
                first(rest),
                Node {
                    prefix: rest.into(),
                    value: Some(value),
                    children: BTreeMap::new(),
                },
            );
            None
        }
    }
    fn remove(&mut self, key: &str) -> Option<V> {
        let rest = strip(key, &self.prefix)?;
        let value = if rest.is_empty() {
            self.value.take()
        } else {
            let c = first(rest).to_string();
            let child = self.children.get_mut(&c)?;
            let v = child.remove(rest);
            if child.value.is_none() && child.children.is_empty() {
                self.children.remove(&c);
            }
            v
        };
        if !self.prefix.is_empty() && self.value.is_none() && self.children.len() == 1 {
            let (_, child) = self.children.pop_first().unwrap();
            self.prefix.push_str(&child.prefix);
            self.value = child.value;
            self.children = child.children;
        }
        value
    }
    fn collect<'a>(&'a self, prefix: &str, out: &mut Vec<(String, &'a V)>) {
        let key = prefix.to_string() + &self.prefix;
        if let Some(v) = &self.value {
            out.push((key.clone(), v))
        }
        for child in self.children.values() {
            child.collect(&key, out)
        }
    }
}
impl<V: Clone> TrieDictionary<V> {
    pub fn subtrie(&self, prefix: &str) -> Self {
        self.traverse(prefix)
    }
    pub fn gathering_values_along_paths<S: AsRef<str>>(
        &self,
        paths: &[S],
    ) -> TrieDictionary<Vec<V>> {
        paths
            .iter()
            .filter_map(|p| {
                let p = p.as_ref();
                let values: Vec<_> = self.values_along_path(p).into_iter().cloned().collect();
                (!values.is_empty()).then(|| (p.to_owned(), values))
            })
            .collect()
    }
    pub fn setting_many(&self, pairs: impl IntoIterator<Item = (String, V)>) -> Self {
        let mut out = self.clone();
        for (k, v) in pairs {
            out.insert(k, v);
        }
        out
    }
    pub fn updating_value(&self, key: &str, value: V) -> (Self, Option<V>) {
        let mut out = self.clone();
        let old = out.insert(key, value);
        (out, old)
    }
    pub fn removing_value(&self, key: &str) -> (Self, Option<V>) {
        let mut out = self.clone();
        let old = out.remove(key);
        (out, old)
    }
    pub fn removing_many<S: AsRef<str>>(&self, keys: impl IntoIterator<Item = S>) -> Self {
        let mut out = self.clone();
        for k in keys {
            out.remove(k.as_ref());
        }
        out
    }
    pub fn removing_all(&self) -> Self {
        Self::new()
    }
    pub fn removing_where(&self, mut f: impl FnMut(&str, &V) -> bool) -> Self {
        self.filter(|k, v| !f(k, v))
    }
    pub fn filtering_keys(&self, mut f: impl FnMut(&str) -> bool) -> Self {
        self.filter(|k, _| f(k))
    }
    pub fn filtering_values(&self, mut f: impl FnMut(&V) -> bool) -> Self {
        self.filter(|_, v| f(v))
    }
    pub fn with_prefix(&self, prefix: &str) -> Self {
        self.filtering_keys(|k| strip(k, prefix).is_some())
    }
    pub fn with_suffix(&self, suffix: &str) -> Self {
        let suffix: String = suffix.nfc().collect();
        self.filtering_keys(|k| k.nfc().collect::<String>().ends_with(&suffix))
    }
    pub fn partitioned(&self, mut f: impl FnMut(&str, &V) -> bool) -> (Self, Self) {
        let (mut yes, mut no) = (Self::new(), Self::new());
        for (k, v) in self.iter() {
            if f(&k, v) {
                yes.insert(k, v.clone());
            } else {
                no.insert(k, v.clone());
            }
        }
        (yes, no)
    }
    pub fn replacing_values<U>(&self, mut f: impl FnMut(&str, &V) -> U) -> TrieDictionary<U> {
        self.iter()
            .into_iter()
            .map(|(k, v)| {
                let value = f(&k, v);
                (k, value)
            })
            .collect()
    }
    pub fn filtering_key_length(&self, mut f: impl FnMut(usize) -> bool) -> Self {
        self.filtering_keys(|k| f(k.graphemes(true).count()))
    }
    pub fn with_min_key_length(&self, n: usize) -> Self {
        self.filtering_key_length(|len| len >= n)
    }
    pub fn with_max_key_length(&self, n: usize) -> Self {
        self.filtering_key_length(|len| len <= n)
    }
    pub fn with_key_length(&self, n: usize) -> Self {
        self.filtering_key_length(|len| len == n)
    }
    pub fn transform_and_filter<U>(
        &self,
        mut f: impl FnMut(&str, &V) -> Option<(String, U)>,
    ) -> TrieDictionary<U> {
        self.iter()
            .into_iter()
            .filter_map(|(k, v)| f(&k, v))
            .collect()
    }
}
impl<V: Clone + std::hash::Hash + Eq> TrieDictionary<V> {
    pub fn unique_values(&self) -> Self {
        let mut seen = std::collections::HashSet::new();
        self.filter(|_, v| seen.insert(v.clone()))
    }
}
impl<'a, V> IntoIterator for &'a TrieDictionary<V> {
    type Item = (String, &'a V);
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter().into_iter()
    }
}
impl<V: PartialEq> PartialEq for TrieDictionary<V> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .into_iter()
                .all(|(k, v)| other.get(&k) == Some(v))
    }
}
impl<V: Eq> Eq for TrieDictionary<V> {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn split_collapse_persistence() {
        let mut t = TrieDictionary::new();
        for k in ["apple", "application", "ape", "bat", ""] {
            t.insert(k, k.len());
        }
        let old = t.clone();
        assert_eq!(t.get("application"), Some(&11));
        t.remove("ape");
        t.remove("apple");
        assert_eq!(t.get("application"), Some(&11));
        assert_eq!(old.len(), 5);
        assert!(t.is_fully_compressed());
        assert_eq!(t.traverse("app").keys(), vec!["lication"]);
    }
    #[test]
    fn model_sequence() {
        let mut t = TrieDictionary::new();
        let mut m = BTreeMap::new();
        for i in 0..1000 {
            let key = format!("{}-{}", i % 7, i % 31);
            if i % 3 == 0 {
                assert_eq!(t.remove(&key), m.remove(&key));
            } else {
                assert_eq!(t.insert(key.clone(), i), m.insert(key, i));
            }
            assert_eq!(
                t.iter(),
                m.iter().map(|(k, v)| (k.clone(), v)).collect::<Vec<_>>()
            );
            assert!(t.is_fully_compressed());
        }
    }
}
