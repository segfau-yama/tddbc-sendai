use rust_molkky::{
    Id, Molkky, Name, Skittle, SkittleState, Team, Throw,
    gui_2d::{
        models::{GuiSkittle, GuiStick},
        services::GuiMolkky,
    },
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
    GuiStick::new(*stick.stick(), Some(target), stick.speed()).throw(speed)
}

fn settle(rules: &mut GuiMolkky, stick: &GuiStick) {
    for _ in 0..(20.0 / GuiMolkky::TIME_STEP) as usize {
        assert!(rules.step(stick));
        if rules.is_settled() {
            return;
        }
    }
    panic!("20秒相当の更新で停止しませんでした");
}

mod launching {
    use super::*;

    #[test]
    fn 狙う候補を選び速度を与えて一刻みだけ移動する() {
        let (mut rules, stick, pins) = scene(&[[0.0, 1.0], [0.2, 1.0], [-0.1, -0.1]]);
        let aimed = shot(&stick, [0.0, 1.0], 6.0);
        assert_eq!(rules.aim(&aimed, &pins), [pins[0]]);
        assert_eq!(rules.pose(stick.stick().id()).unwrap().0, [0.0, 0.0]);
        assert!(rules.step(&aimed));
        assert!(!rules.is_settled());
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
    fn 無効な投擲は進まず計算中は初速を上書きしない() {
        let (mut rules, stick, _) = scene(&[]);
        for invalid in [
            stick,
            shot(&stick, [0.0, 0.0], 6.0),
            shot(&stick, [f32::NAN, 1.0], 6.0),
            shot(&stick, [0.0, 1.0], 0.0),
            shot(&stick, [0.0, 1.0], -1.0),
            shot(&stick, [0.0, 1.0], f32::NAN),
            shot(&stick, [0.0, 1.0], f32::INFINITY),
            shot(&GuiStick::default(), [0.0, 1.0], 6.0),
        ] {
            assert!(!rules.step(&invalid));
            assert!(rules.is_settled());
            assert_eq!(rules.pose(stick.stick().id()), Some(([0.0, 0.0], 0.0)));
        }
        let aimed = shot(&stick, [0.0, 1.0], 6.0);
        assert!(rules.step(&aimed));
        let before = rules.pose(stick.stick().id()).unwrap().0;
        assert!(rules.step(&shot(&stick, [1.0, 0.0], 3.0)));
        let (after, angle) = rules.pose(stick.stick().id()).unwrap();
        assert!(after[0].abs() < 0.001);
        assert!(after[1] > before[1]);
        assert!(after[1] - before[1] < before[1]);
        assert!(angle.abs() < 0.001);
        settle(&mut rules, &aimed);
        let stopped = rules.pose(stick.stick().id());
        assert!(rules.step(&aimed));
        assert!(rules.is_settled());
        assert_eq!(rules.pose(stick.stick().id()), stopped);
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
    fn 単体hitは物理を進めず更新した転倒状態を返す() {
        let (mut rules, stick, pins) = scene(&[[0.0, 0.7]]);
        let aimed = shot(&stick, [0.0, 1.0], 6.0);
        let pose = rules.pose(pins[0].id());
        assert_eq!(rules.hit(&aimed, &pins[0]), pins[0]);
        assert_eq!(rules.pose(pins[0].id()), pose);
        settle(&mut rules, &aimed);
        let pose = rules.pose(pins[0].id());
        let updated = rules.hit(&aimed, &pins[0]);
        assert_eq!(updated, pins[0].with_state(SkittleState::Fall));
        assert_eq!(rules.hit(&aimed, &updated), updated);
        assert_eq!(rules.hit(&GuiStick::default(), &pins[0]), pins[0]);
        assert_eq!(rules.pose(pins[0].id()), pose);
    }

    #[test]
    fn 立て直しは位置と個体を保ち今回の転倒と速度と回転をクリアする() {
        let (mut rules, stick, pins) = scene(&[[0.0, 0.7]]);
        let aimed = shot(&stick, [0.0, 1.0], 6.0);
        settle(&mut rules, &aimed);
        assert_eq!(rules.hit(&aimed, &pins[0]).state(), SkittleState::Fall);
        let position = rules.pose(pins[0].id()).unwrap().0;
        rules.reset_throw();
        assert_eq!(rules.pose(pins[0].id()), Some((position, 0.0)));
        assert_eq!(rules.pose(stick.stick().id()), Some(([0.0, 0.0], 0.0)));
        assert_eq!(rules.hit(&aimed, &pins[0]), pins[0]);
        assert!(!rules.step(&stick));
        assert!(rules.is_settled());
        assert_eq!(rules.pose(pins[0].id()), Some((position, 0.0)));
    }

    #[test]
    fn 高速投擲でもすり抜けず同番号の個体を区別して倒す() {
        let (mut rules, stick, pins) = scene(&[[0.0, 1.0], [0.2, 1.0]]);
        let aimed = shot(&stick, [0.0, 1.0], 120.0);
        // 反射後に別の個体へ当たる場合と区別し、最初の衝突を確認する。
        for _ in 0..(20.0 / GuiMolkky::TIME_STEP) as usize {
            assert!(rules.step(&aimed));
            if !fallen_skittles(&rules, &aimed, &pins).is_empty() {
                break;
            }
        }
        let fallen = fallen_skittles(&rules, &aimed, &pins);
        assert_eq!(fallen, [pins[0].with_state(SkittleState::Fall)]);
        assert_eq!(fallen_skittles(&rules, &aimed, &pins), fallen);
        assert!(fallen_skittles(&rules, &GuiStick::default(), &pins).is_empty());
        assert!(pins.iter().all(|p| p.state() == SkittleState::Stand));
        settle(&mut rules, &aimed);
    }

    #[test]
    fn 小さな衝突は移動しても転倒に数えない() {
        // 0.1m/sを減衰させた移動距離でも接触する位置に置く。
        let (mut rules, stick, pins) = scene(&[[0.0, 0.16]]);
        let aimed = shot(&stick, [0.0, 1.0], 0.1);
        settle(&mut rules, &aimed);
        assert!(fallen_skittles(&rules, &aimed, &pins).is_empty());
        assert!(rules.pose(pins[0].id()).unwrap().0[1] > 0.16);
    }

    #[test]
    fn ピン同士の衝突も今回の転倒へ含める() {
        let (mut rules, stick, pins) = scene(&[[0.02, 0.7], [0.06, 0.85]]);
        let aimed = shot(&stick, [0.0, 1.0], 6.0);
        assert_eq!(rules.aim(&aimed, &pins), [pins[0]]);
        settle(&mut rules, &aimed);
        let fallen = fallen_skittles(&rules, &aimed, &pins);
        assert_eq!(
            fallen.len(),
            2,
            "{:?}",
            pins.iter().map(|p| rules.pose(p.id())).collect::<Vec<_>>()
        );
        assert_ne!(fallen[0].id(), fallen[1].id());
    }

    #[test]
    fn chain_hitは確率を使わず物理の連鎖結果を反映しstand_upは個体だけを立てる() {
        let (mut rules, stick, pins) = scene(&[[0.02, 0.7], [0.06, 0.85]]);
        let aimed = shot(&stick, [0.0, 1.0], 6.0);
        assert_eq!(
            rules.chain_hit(&pins[0].with_state(SkittleState::Fall), &pins[1]),
            pins[1]
        );
        settle(&mut rules, &aimed);
        let fallen = rules.hit(&aimed, &pins[0]);
        let chained = rules.chain_hit(&fallen, &pins[1]);
        assert_eq!(chained, pins[1].with_state(SkittleState::Fall));
        assert_eq!(rules.chain_hit(&pins[0], &pins[1]), pins[1]);
        let pose = rules.pose(chained.id());
        assert_eq!(rules.stand_up(&chained), pins[1]);
        assert_eq!(rules.pose(chained.id()), pose);
        assert_eq!(chained.state(), SkittleState::Fall);
    }

    #[test]
    fn 次の投擲で位置を保持し前回の転倒をミスへ持ち越さない() {
        let (mut rules, stick, pins) = scene(&[[0.0, 0.7], [0.04, 0.9]]);
        let aimed = shot(&stick, [0.0, 1.0], 6.0);
        settle(&mut rules, &aimed);
        assert!(!fallen_skittles(&rules, &aimed, &pins).is_empty());
        let positions: Vec<_> = pins.iter().map(|p| rules.pose(p.id()).unwrap().0).collect();
        // 後ろの壁で反射して当たる投擲ではなく、届かない弱い投擲でミスにする。
        rules.reset_throw();
        let missed = shot(&stick, [1.0, 0.0], 0.1);
        assert!(fallen_skittles(&rules, &aimed, &pins).is_empty());
        assert_eq!(
            pins.iter()
                .map(|p| rules.pose(p.id()).unwrap().0)
                .collect::<Vec<_>>(),
            positions
        );
        settle(&mut rules, &missed);
        assert!(fallen_skittles(&rules, &aimed, &pins).is_empty());
        assert!(rules.step(&missed));
        assert!(rules.is_settled());
    }
}

mod field_boundaries {
    use super::*;

    fn assert_bounces(target: [f32; 2], speed: f32) {
        let (mut rules, stick, _) = scene(&[]);
        let aimed = shot(&stick, target, speed);
        let mut previous = [0.0, 0.0];
        let mut reflected = [false; 2];
        for _ in 0..(20.0 / GuiMolkky::TIME_STEP) as usize {
            assert!(rules.step(&aimed));
            let (position, angle) = rules.pose(stick.stick().id()).unwrap();
            let half_segment = GuiStick::LENGTH / 2.0 - GuiStick::RADIUS;
            let extents = [angle.sin().abs(), angle.cos().abs()]
                .map(|component| GuiStick::RADIUS + half_segment * component);
            for (axis, min, max) in [(0, -0.3, 0.3), (1, -0.15, 1.35)] {
                assert!(
                    position[axis] - extents[axis] >= min - 0.005
                        && position[axis] + extents[axis] <= max + 0.005,
                    "棒がフィールドから出ました: {position:?}, {angle}"
                );
                if (position[axis] - previous[axis]) * target[axis] < -0.00001 {
                    reflected[axis] = true;
                }
            }
            previous = position;
            if rules.is_settled() {
                break;
            }
        }
        for axis in 0..2 {
            if target[axis] != 0.0 {
                assert!(reflected[axis], "壁で反射しませんでした: {target:?}");
            }
        }
        assert!(rules.is_settled(), "反射後に停止しませんでした");
    }

    #[test]
    fn 四辺で棒が反射し範囲内で停止する() {
        for target in [[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]] {
            assert_bounces(target, 10.0);
        }
    }

    #[test]
    fn 高速投擲でも外周の壁をすり抜けない() {
        for target in [[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]] {
            assert_bounces(target, 120.0);
        }
    }

    #[test]
    fn 角へ斜めに投げても両方の方向へ反射して停止する() {
        for target in [[0.3, 1.35], [-0.3, 1.35], [0.3, -0.15], [-0.3, -0.15]] {
            assert_bounces(target, 10.0);
        }
    }

    #[test]
    fn 強く打たれたスキットルも外周の内側に残る() {
        let (mut rules, stick, skittles) = scene(&[[0.0, 0.7]]);
        let aimed = shot(&stick, [0.0, 1.0], 120.0);
        for _ in 0..(20.0 / GuiMolkky::TIME_STEP) as usize {
            assert!(rules.step(&aimed));
            let [x, y] = rules.pose(skittles[0].id()).unwrap().0;
            let radius = GuiSkittle::RADIUS;
            assert!(
                x - radius >= -0.305
                    && x + radius <= 0.305
                    && y - radius >= -0.155
                    && y + radius <= 1.355,
                "スキットルがフィールドから出ました: {x}, {y}"
            );
            if rules.is_settled() {
                break;
            }
        }
        assert!(rules.is_settled());
        assert_eq!(
            fallen_skittles(&rules, &aimed, &skittles),
            [skittles[0].with_state(SkittleState::Fall)]
        );
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

fn fallen_skittles<M: Molkky + ?Sized>(
    rules: &M,
    stick: &M::Stick,
    skittles: &[M::Skittle],
) -> Vec<Skittle> {
    skittles
        .iter()
        .map(|skittle| rules.hit(stick, skittle))
        .filter(|skittle| skittle.as_ref().state() == SkittleState::Fall)
        .map(|skittle| *skittle.as_ref())
        .collect()
}
