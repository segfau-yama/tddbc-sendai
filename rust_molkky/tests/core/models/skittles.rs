use rust_molkky::{Skittle, Skittles};

#[test]
fn 一時的なモデルの破棄後も元のスライスを参照できる() {
    let pins = [Skittle::new(5)];
    let borrowed = Skittles::new(&pins).as_slice();
    assert!(std::ptr::eq(borrowed, &pins[..]));
}

#[test]
fn 生成時に番号でなく個体の重複を判定し入力を変えない() {
    let pins = [Skittle::new(12), Skittle::new(12)];
    let original = pins;
    Skittles::new(&[]);
    Skittles::new(&pins);
    assert_eq!(pins, original);
    assert!(
        std::panic::catch_unwind(|| {
            Skittles::new(&[pins[0], pins[0]]);
        })
        .is_err()
    );
}

#[test]
#[should_panic(expected = "スキットルIDが重複しています")]
fn 同じ個体は状態が異なっても二重登録できない() {
    let pin = Skittle::new(5);
    Skittles::new(&[pin, pin.with_state(rust_molkky::SkittleState::Fall)]);
}

mod scoring {
    use rust_molkky::Skittle;

    mod one_skittle {
        use super::Skittle;

        #[test]
        fn 番号5のスキットルを倒すと5点になる() {
            assert_eq!(
                rust_molkky::Skittles::new(&[Skittle::new(5)])
                    .calc_score()
                    .value(),
                5
            );
        }

        #[test]
        fn 番号1のスキットルを倒すと1点になる() {
            assert_eq!(
                rust_molkky::Skittles::new(&[Skittle::new(1)])
                    .calc_score()
                    .value(),
                1
            );
        }

        #[test]
        fn 番号12のスキットルを倒すと12点になる() {
            assert_eq!(
                rust_molkky::Skittles::new(&[Skittle::new(12)])
                    .calc_score()
                    .value(),
                12
            );
        }
    }

    mod multiple_skittles {
        use super::Skittle;

        #[test]
        fn 複数本倒すと倒した本数が点数になる() {
            assert_eq!(
                rust_molkky::Skittles::new(&[Skittle::new(5), Skittle::new(8)])
                    .calc_score()
                    .value(),
                2
            );
        }

        #[test]
        fn 番号1と5と12の3本を倒すと3点になる() {
            assert_eq!(
                rust_molkky::Skittles::new(&[Skittle::new(1), Skittle::new(5), Skittle::new(12)])
                    .calc_score()
                    .value(),
                3
            );
        }

        #[test]
        fn 同じ番号の別スキットルは本数で採点する() {
            assert_eq!(
                rust_molkky::Skittles::new(&[Skittle::new(5), Skittle::new(5)])
                    .calc_score()
                    .value(),
                2
            );
        }

        #[test]
        #[should_panic(expected = "スキットルIDが重複しています")]
        fn 同じ個体のコピーを重複として拒否する() {
            let pin = Skittle::new(5);
            rust_molkky::Skittles::new(&[pin, pin]).calc_score();
        }
    }

    mod no_skittles {

        #[test]
        fn 一本も倒さなければ0点になる() {
            assert_eq!(rust_molkky::Skittles::new(&[]).calc_score().value(), 0);
        }
    }

    mod immutability {
        use super::Skittle;

        #[test]
        fn 元の配列を変更しても点数は変わらない() {
            let mut skittles = [Skittle::new(5)];
            let thrown = rust_molkky::Skittles::new(&skittles).calc_score();

            skittles[0] = Skittle::new(12);

            assert_eq!(skittles[0].number(), 12);
            assert_eq!(thrown.value(), 5);
        }
    }

    #[test]
    fn 標準の十二本に別個体の十二番を追加した十三本を採点できる() {
        let mut pins: Vec<_> = (1..=12).map(Skittle::new).collect();
        pins.push(Skittle::new(12));
        assert_eq!(rust_molkky::Skittles::new(&pins).calc_score().value(), 13);
    }
}
