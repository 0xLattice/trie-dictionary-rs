use lattice_trie_dictionary::TrieDictionary;
#[test]
fn persistent_extensions_and_unicode_lengths() {
    let t: TrieDictionary<_> = [
        ("a".to_owned(), 1),
        ("ab".to_owned(), 2),
        ("é".to_owned(), 3),
        ("b".to_owned(), 2),
    ]
    .into_iter()
    .collect();
    let (updated, old) = t.updating_value("e\u{301}", 4);
    assert_eq!(old, Some(3));
    assert_eq!(t.get("é"), Some(&3));
    assert_eq!(updated.get("é"), Some(&4));
    assert_eq!(t.with_prefix("a").len(), 2);
    assert_eq!(t.with_key_length(1).len(), 3);
    assert_eq!(t.with_suffix("e\u{301}").len(), 1);
    let (yes, no) = t.partitioned(|_, v| *v == 2);
    assert_eq!((yes.len(), no.len()), (2, 2));
    assert_eq!(yes.merge(&no, |_, _| unreachable!()), t);
    assert_eq!(t.unique_values().len(), 3);
    let gathered = t.gathering_values_along_paths(&["abc", "missing"]);
    assert_eq!(gathered.get("abc"), Some(&vec![1, 2]));
    assert!(gathered.get("missing").is_none());
    assert_eq!(t.subtrie("a").get("b"), Some(&2));
    assert_eq!(t.removing_many(["a", "ab"]).len(), 2);
}
