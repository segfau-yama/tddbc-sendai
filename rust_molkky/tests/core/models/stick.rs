use rust_molkky::Stick;

mod identity {
    use super::*;

    #[test]
    fn 新しい個体は異なる識別子を持ちコピーは同じ識別子を保つ() {
        let first = Stick::new();
        let second = Stick::default();
        assert_ne!(first.id(), second.id());
        let copied = first;
        assert_eq!(copied.id(), first.id());
    }
}
