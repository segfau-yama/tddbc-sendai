use rust_molkky::{Skittle, Throw};

mod one_skittle {
    use super::{Skittle, Throw};

    #[test]
    fn 番号5のスキットルを倒すと5点になる() {
        assert_eq!(Throw::new(&[Skittle::new(5)]).calc_point(), 5);
    }

    #[test]
    fn 番号1のスキットルを倒すと1点になる() {
        assert_eq!(Throw::new(&[Skittle::new(1)]).calc_point(), 1);
    }

    #[test]
    fn 番号12のスキットルを倒すと12点になる() {
        assert_eq!(Throw::new(&[Skittle::new(12)]).calc_point(), 12);
    }
}

mod multiple_skittles {
    use super::{Skittle, Throw};

    #[test]
    fn 複数本倒すと倒した本数が点数になる() {
        assert_eq!(
            Throw::new(&[Skittle::new(5), Skittle::new(8)]).calc_point(),
            2
        );
    }

    #[test]
    fn 番号1と5と12の3本を倒すと3点になる() {
        assert_eq!(
            Throw::new(&[Skittle::new(1), Skittle::new(5), Skittle::new(12)]).calc_point(),
            3
        );
    }

    #[test]
    #[should_panic(expected = "スキットル番号が重複しています: 5")]
    fn 同じ番号の別スキットルを重複として拒否する() {
        Throw::new(&[Skittle::new(5), Skittle::new(5)]);
    }
}

mod no_skittles {
    use super::Throw;

    #[test]
    fn 一本も倒さなければ0点になる() {
        assert_eq!(Throw::new(&[]).calc_point(), 0);
    }
}

mod immutability {
    use super::{Skittle, Throw};

    #[test]
    fn 元の配列を変更しても点数は変わらない() {
        let mut skittles = [Skittle::new(5)];
        let thrown = Throw::new(&skittles);

        skittles[0] = Skittle::new(12);

        assert_eq!(skittles[0].number(), 12);
        assert_eq!(thrown.calc_point(), 5);
    }
}
