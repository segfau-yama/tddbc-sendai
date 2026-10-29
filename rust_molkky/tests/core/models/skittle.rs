use rust_molkky::{Skittle, SkittleState};

mod valid_numbers {
    use super::*;

    #[test]
    fn 番号の下限一を保持する() {
        assert_eq!(Skittle::new(1).number(), 1);
    }

    #[test]
    fn 番号の上限十二を保持する() {
        assert_eq!(Skittle::new(12).number(), 12);
    }
}

mod invalid_numbers {
    use super::*;

    #[test]
    #[should_panic(expected = "スキットル番号は1〜12です")]
    fn 番号零を拒否する() {
        Skittle::new(0);
    }

    #[test]
    #[should_panic(expected = "スキットル番号は1〜12です")]
    fn 番号十三を拒否する() {
        Skittle::new(13);
    }

    #[test]
    #[should_panic(expected = "スキットル番号は1〜12です")]
    fn 一覧の途中に不正な番号があっても生成できない() {
        let _pins = [Skittle::new(1), Skittle::new(13)];
    }
}

mod identity {
    use super::*;

    #[test]
    fn 同じ番号でも個体は異なりコピーは同じ識別子を保つ() {
        let first = Skittle::new(12);
        let second = Skittle::new(12);
        assert_ne!(first.id(), second.id());
        let copied = first;
        assert_eq!(copied.id(), first.id());
    }
}

mod state {
    use super::*;

    #[test]
    fn 状態を更新しても元の状態と識別子と番号を保つ() {
        let original = Skittle::new(12);
        let updated = original.with_state(SkittleState::Fall);
        assert_eq!(original.state(), SkittleState::Stand);
        assert_eq!(updated.state(), SkittleState::Fall);
        assert_eq!(updated.id(), original.id());
        assert_eq!(updated.number(), original.number());
    }
}
