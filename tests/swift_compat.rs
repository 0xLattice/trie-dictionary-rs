use lattice_trie_dictionary::TrieDictionary;
#[test]
fn swift_mutation_vectors() {
    let mut t = TrieDictionary::new();
    let expected: Vec<_> = include_str!("swift.txt").lines().collect();
    for (i, expected_line) in expected.iter().enumerate().take(100) {
        let key = format!("{}-{}", i % 7, i % 11);
        if i % 3 == 0 {
            t.remove(&key);
        } else {
            t.insert(key, i);
        }
        let actual = t
            .iter()
            .into_iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(actual, *expected_line, "step {i}");
    }
}
#[test]
fn unicode_canonical_equivalence() {
    let mut t = TrieDictionary::new();
    t.insert("é", 1);
    assert_eq!(t.get("e\u{301}"), Some(&1));
    assert_eq!(t.insert("e\u{301}", 2), Some(1));
    assert_eq!(t.len(), 1);
    t.insert("éx", 3);
    assert_eq!(t.get("e\u{301}x"), Some(&3));
    assert_eq!(t.remove("e\u{301}"), Some(2));
    assert_eq!(t.get("éx"), Some(&3));
}
