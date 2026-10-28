use rust_molkky::Id;

#[test]
fn 新しい識別子はuuid_v4でありコピーしても同じ値になる() {
    let first = Id::new();
    let second = Id::new();
    assert_ne!(first, second);
    let copied = first;
    assert_eq!(first, copied);
    let uuid = uuid::Uuid::parse_str(&first.to_string()).unwrap();
    assert_eq!(uuid.get_version_num(), 4);
    assert!(!uuid.is_nil());
    let mut ids = std::collections::HashSet::new();
    ids.insert(first);
    assert!(ids.contains(&first));
    assert!(!ids.contains(&second));
}
