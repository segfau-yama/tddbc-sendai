use rust_molkky::{
    Id, Molkky, Name, Skittle, SkittleState, Team,
    gui_2d::{models::GuiStick, services::GuiMolkky},
};

fn scene(positions: &[[f32; 2]]) -> (GuiMolkky, GuiStick, Vec<Skittle>) {
    let stick = GuiStick::default();
    let pins: Vec<_> = positions.iter().map(|_| Skittle::new(12)).collect();
    let placement: Vec<_> = pins
        .iter()
        .zip(positions)
        .map(|(pin, &pos)| (pin.id(), pos))
        .collect();
    (GuiMolkky::new(&stick, [0.0, 0.0], &placement), stick, pins)
}

fn shot(stick: &GuiStick, target: [f32; 2], speed: f32) -> GuiStick {
    GuiStick::new(*stick.stick(), Some(target), speed)
}

fn settle(rules: &mut GuiMolkky) {
    for _ in 0..1200 {
        if rules.is_settled() {
            return;
        }
        rules.step();
    }
    panic!("20秒相当の更新で停止しませんでした");
}

mod launching {
    use super::*;

    #[test]
    fn 狙う候補を選び速度を与えて一刻みだけ移動する() {
        let (mut rules, stick, pins) = scene(&[[0.0, 1.0], [1.0, 1.0], [0.0, -1.0]]);
        let aimed = shot(&stick, [0.0, 1.0], 6.0);
        assert_eq!(rules.aim(&aimed, &pins), [pins[0]]);
        assert!(rules.throw(&aimed));
        assert!(!rules.is_settled());
        assert_eq!(rules.pose(stick.stick().id()).unwrap().0, [0.0, 0.0]);
        rules.step();
        let (pos, angle) = rules.pose(stick.stick().id()).unwrap();
        assert!(pos[1] > 0.0 && pos[1] < 0.11, "{pos:?}");
        assert!(pos[0].abs() < 0.001);
        assert!(angle.abs() < 0.001);
        assert!(rules.pose(Id::new()).is_none());
        for pin in &pins {
            assert!(rules.pose(pin.id()).is_some());
        }
    }

    #[test]
    fn 無効な投擲と計算中の再投擲は状態を変えない() {
        let (mut rules, stick, _) = scene(&[]);
        for invalid in [
            stick,
            shot(&stick, [0.0, 0.0], 6.0),
            shot(&stick, [f32::NAN, 1.0], 6.0),
            shot(&stick, [0.0, 1.0], 0.0),
            shot(&stick, [0.0, 1.0], -1.0),
            shot(&stick, [0.0, 1.0], f32::INFINITY),
            shot(&GuiStick::default(), [0.0, 1.0], 6.0),
        ] {
            assert!(!rules.throw(&invalid));
            assert!(rules.is_settled());
        }
        assert!(rules.throw(&shot(&stick, [0.0, 1.0], 6.0)));
        rules.step();
        let before = rules.pose(stick.stick().id());
        assert!(!rules.throw(&shot(&stick, [1.0, 0.0], 3.0)));
        assert_eq!(rules.pose(stick.stick().id()), before);
    }

    #[test]
    fn 不正な配置を物理ワールドへ渡さない() {
        let stick = GuiStick::default();
        let id = Id::new();
        for placement in [
            vec![(id, [0.0, 1.0]), (id, [1.0, 1.0])],
            vec![(stick.stick().id(), [0.0, 1.0])],
            vec![(id, [f32::INFINITY, 1.0])],
        ] {
            assert!(
                std::panic::catch_unwind(|| GuiMolkky::new(&stick, [0.0, 0.0], &placement))
                    .is_err()
            );
        }
        assert!(std::panic::catch_unwind(|| GuiMolkky::new(&stick, [f32::NAN, 0.0], &[])).is_err());
    }
}

mod collisions {
    use super::*;

    #[test]
    fn 高速投擲でもすり抜けず同番号の個体を区別して倒す() {
        let (mut rules, stick, pins) = scene(&[[0.0, 1.0], [3.0, 1.0]]);
        let aimed = shot(&stick, [0.0, 1.0], 120.0);
        assert!(rules.throw(&aimed));
        settle(&mut rules);
        let fallen = rules.hit(&aimed, &pins);
        assert_eq!(fallen, [pins[0].with_state(SkittleState::Fall)]);
        assert_eq!(rules.hit(&aimed, &pins), fallen);
        assert!(rules.hit(&GuiStick::default(), &pins).is_empty());
        assert!(pins.iter().all(|p| p.state() == SkittleState::Stand));
    }

    #[test]
    fn 小さな衝突は移動しても転倒に数えない() {
        // 0.1m/sを減衰させた移動距離でも接触する位置に置く。
        let (mut rules, stick, pins) = scene(&[[0.0, 0.16]]);
        let aimed = shot(&stick, [0.0, 1.0], 0.1);
        assert!(rules.throw(&aimed));
        settle(&mut rules);
        assert!(rules.hit(&aimed, &pins).is_empty());
        assert!(rules.pose(pins[0].id()).unwrap().0[1] > 0.16);
    }

    #[test]
    fn ピン同士の衝突も今回の転倒へ含める() {
        let (mut rules, stick, pins) = scene(&[[0.02, 0.7], [0.06, 0.85]]);
        let aimed = shot(&stick, [0.0, 1.0], 6.0);
        assert_eq!(rules.aim(&aimed, &pins), [pins[0]]);
        assert!(rules.throw(&aimed));
        settle(&mut rules);
        let fallen = rules.hit(&aimed, &pins);
        assert_eq!(
            fallen.len(),
            2,
            "{:?}",
            pins.iter().map(|p| rules.pose(p.id())).collect::<Vec<_>>()
        );
        assert_ne!(fallen[0].id(), fallen[1].id());
    }

    #[test]
    fn 次の投擲で位置を保持し前回の転倒をミスへ持ち越さない() {
        let (mut rules, stick, pins) = scene(&[[0.0, 0.7], [0.04, 0.9]]);
        let aimed = shot(&stick, [0.0, 1.0], 6.0);
        assert!(rules.throw(&aimed));
        settle(&mut rules);
        assert!(!rules.hit(&aimed, &pins).is_empty());
        let positions: Vec<_> = pins.iter().map(|p| rules.pose(p.id()).unwrap().0).collect();
        assert!(rules.throw(&shot(&stick, [0.0, -1.0], 6.0)));
        assert!(rules.hit(&aimed, &pins).is_empty());
        assert_eq!(
            pins.iter()
                .map(|p| rules.pose(p.id()).unwrap().0)
                .collect::<Vec<_>>(),
            positions
        );
        settle(&mut rules);
        assert!(rules.hit(&aimed, &pins).is_empty());
        rules.step();
        assert!(rules.is_settled());
    }
}

mod core_rules {
    use super::*;

    #[test]
    fn 採点と勝者と次手番にはcoreのルールを使う() {
        let (rules, _, _) = scene(&[]);
        assert_eq!(rules.calc_score(&[Skittle::new(12)]).value(), 12);
        assert_eq!(
            rules
                .calc_score(&[Skittle::new(5), Skittle::new(8)])
                .value(),
            2
        );
        let points = (0..3).map(|_| rules.calc_score(&[])).collect();
        let disqualified = rust_molkky::ScoreBoard::new(points);
        let teams = [
            Team::new(Name::new("赤")).with_turn(true),
            Team::new(Name::new("青")),
        ];
        assert_eq!(rules.next_team(&teams).unwrap().id(), teams[1].id());
        let teams = [teams[0].with_score_board(disqualified), teams[1].clone()];
        assert_eq!(rules.judge_game(&teams).unwrap().id(), teams[1].id());
        assert!(rules.next_team(&teams).is_none());
    }
}
