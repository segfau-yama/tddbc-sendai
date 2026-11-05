use crate::{
    Molkky, Move, Name, Skittle, SkittleState, StandUp, StickState, Team,
    gui_2d::{
        models::{GuiSkittle, GuiStick},
        services::{GuiMolkky, TIME_STEP, field::create_world},
    },
};
use rapier2d::prelude::{Rotation, Vector};

// サービスの契約を描画から独立して確認するための実エンジンfixture。
// Fieldの接触振り分けを複製せず、各hit/chain_hitへ結果を渡す。
fn advance(
    rules: &mut GuiMolkky,
    skittles: &[GuiSkittle],
    stick: GuiStick,
) -> (Vec<GuiSkittle>, GuiStick) {
    if stick.as_ref().state() == StickState::Held {
        return (skittles.to_vec(), stick);
    }
    let body = &mut rules.world.bodies[rules.stick_body];
    if body.translation().to_array() != stick.position()
        || body.linvel().to_array() != stick.velocity()
        || body.rotation().angle() != stick.angle()
        || body.angvel() != stick.angular_velocity()
    {
        body.set_translation(Vector::from_array(stick.position()), false);
        body.set_rotation(Rotation::from_angle(stick.angle()), false);
        body.set_linvel(Vector::from_array(stick.velocity()), false);
        body.set_angvel(stick.angular_velocity(), false);
        body.wake_up(true);
    }
    rules.world.step();
    for pair in rules.world.narrow_phase.contact_pairs() {
        let [a, b] = [pair.collider1, pair.collider2];
        if [a, b].iter().all(|handle| {
            rules.world.colliders[*handle]
                .parent()
                .is_none_or(|body| rules.world.bodies[body].is_sleeping())
        }) {
            continue;
        }
        for key in [(a, b), (b, a)] {
            let peak = rules.impacts.entry(key).or_default();
            *peak = peak.max(pair.total_impulse_magnitude());
        }
    }
    let body = &rules.world.bodies[rules.stick_body];
    let stick = stick.update(
        body.translation().to_array(),
        body.rotation().angle(),
        if body.is_sleeping() {
            [0.0; 2]
        } else {
            body.linvel().to_array()
        },
        if body.is_sleeping() {
            0.0
        } else {
            body.angvel()
        },
    );
    let updated = skittles
        .iter()
        .map(|target| {
            let mut result = rules.hit(&stick, target);
            for source in skittles {
                result = rules.chain_hit(source, &result);
            }
            result
        })
        .collect();
    (updated, stick)
}

fn scene(positions: &[[f32; 2]]) -> (GuiMolkky, Vec<GuiSkittle>, GuiStick) {
    let stick = GuiStick::default();
    let skittles: Vec<_> = positions
        .iter()
        .map(|&pos| GuiSkittle::new(Skittle::new(12), pos))
        .collect();
    (create_world(&stick, &skittles), skittles, stick)
}

fn shot(stick: &GuiStick, target: [f32; 2], speed: f32) -> GuiStick {
    GuiStick::r#move(GuiStick::new(*stick.as_ref(), Some(target), speed))
}

fn settle(rules: &mut GuiMolkky, skittles: &mut Vec<GuiSkittle>, stick: &mut GuiStick) {
    for _ in 0..(20.0 / TIME_STEP) as usize {
        (*skittles, *stick) = advance(rules, skittles, *stick);
        if stick.as_ref().state() == StickState::Stop
            && skittles
                .iter()
                .all(|p| p.as_ref().state() != SkittleState::Hit)
        {
            return;
        }
    }
    panic!("20秒相当の更新で停止しませんでした");
}

fn fallen_skittles(skittles: &[GuiSkittle]) -> Vec<Skittle> {
    skittles
        .iter()
        .map(|p| *p.as_ref())
        .filter(|p| p.state() == SkittleState::Fall)
        .collect()
}

mod movement {
    use super::*;

    #[test]
    fn 個体数によらず共有ワールドを一刻みだけ進める() {
        let (mut empty, empty_skittles, empty_stick) = scene(&[]);
        let positions: Vec<_> = (0..12)
            .map(|index| [0.0, 0.6 + index as f32 * 0.0551])
            .collect();
        let (mut full, full_skittles, full_stick) = scene(&positions);
        let (_, empty_moved) = advance(
            &mut empty,
            &empty_skittles,
            shot(&empty_stick, [0.0, 1.0], 1.0),
        );
        let (_, full_moved) = advance(
            &mut full,
            &full_skittles,
            shot(&full_stick, [0.0, 1.0], 1.0),
        );
        assert_eq!(empty_moved.position(), full_moved.position());
        assert_eq!(empty_moved.velocity(), full_moved.velocity());
        assert!(full_moved.position()[1] > 0.0 && full_moved.position()[1] < 0.017);
    }

    #[test]
    fn 狙う候補をエンジンで選び一刻みだけ移動したモデルを返す() {
        let (mut rules, skittles, held) = scene(&[[0.0, 1.0], [0.2, 1.0], [-0.1, -0.1]]);
        let stick = shot(&held, [0.0, 1.0], 6.0);
        let targets = rules.aim(&stick, &skittles);
        assert_eq!(
            targets.iter().map(|p| p.as_ref().id()).collect::<Vec<_>>(),
            [skittles[0].as_ref().id()]
        );
        assert!(targets[0].aimed());
        let same_conditions = shot(&GuiStick::default(), [0.0, 1.0], 6.0);
        assert_ne!(same_conditions.as_ref().id(), stick.as_ref().id());
        assert_eq!(rules.aim(&same_conditions, &skittles), targets);
        let (updated, moved) = advance(&mut rules, &skittles, stick);
        assert_eq!(stick.position(), [0.0; 2]);
        assert!(moved.position()[1] > 0.0 && moved.position()[1] < 0.11);
        assert!(moved.position()[0].abs() < 0.001);
        assert!(moved.angle().abs() < 0.001);
        assert_eq!(moved.as_ref().state(), StickState::Move);
        assert_eq!(updated, skittles);
    }

    #[test]
    fn 無効な投擲は進まず更新した棒を渡すと初速を再適用せず減速する() {
        let (mut rules, mut skittles, held) = scene(&[]);
        for invalid in [
            held,
            shot(&held, [0.0; 2], 6.0),
            shot(&held, [f32::NAN, 1.0], 6.0),
            shot(&held, [0.0, 1.0], 0.0),
            shot(&held, [0.0, 1.0], -1.0),
            shot(&held, [0.0, 1.0], f32::NAN),
            shot(&held, [0.0, 1.0], f32::INFINITY),
        ] {
            let (_, result) = advance(&mut rules, &skittles, invalid);
            assert_eq!(result.as_ref().state(), StickState::Held);
            assert_eq!(result.position(), [0.0; 2]);
        }
        let stick = shot(&held, [0.0, 1.0], 6.0);
        let (_, moved) = advance(&mut rules, &skittles, stick);
        let (_, next) = advance(&mut rules, &skittles, moved);
        assert!(next.position()[1] > moved.position()[1]);
        assert!(next.position()[1] - moved.position()[1] < moved.position()[1]);
        assert!(next.speed() < moved.speed());
        let mut stopped = next;
        settle(&mut rules, &mut skittles, &mut stopped);
        let (_, again) = advance(&mut rules, &skittles, stopped);
        assert_eq!(again, stopped);
        let new_throw = shot(&stopped, [0.0, 1.0], 6.0);
        assert_eq!(new_throw.position(), [0.0; 2]);
        let (_, next_throw) = advance(&mut rules, &skittles, new_throw);
        assert!(next_throw.position()[1] > 0.0 && next_throw.position()[1] < 0.11);
        assert_eq!(next_throw.as_ref().id(), stopped.as_ref().id());
    }
}

mod collisions {
    use super::*;

    #[test]
    fn hitは更新済みの物理状態を返し物理時間を進めない() {
        let (mut rules, mut skittles, held) = scene(&[[0.0, 0.7]]);
        let original = skittles[0];
        let mut stick = shot(&held, [0.0, 1.0], 6.0);
        assert_eq!(rules.hit(&stick, &original), original);
        let confirmed = GuiSkittle::new(
            original.as_ref().with_state(SkittleState::Fall),
            original.position(),
        );
        assert_eq!(rules.hit(&stick, &confirmed), confirmed);
        settle(&mut rules, &mut skittles, &mut stick);
        let updated = rules.hit(&stick, &original);
        assert_eq!(
            *updated.as_ref(),
            original.as_ref().with_state(SkittleState::Fall)
        );
        assert_eq!(updated.position(), skittles[0].position());
        assert_eq!(rules.hit(&stick, &updated), updated);
        assert_eq!(rules.hit(&GuiStick::default(), &original), updated);
        assert_eq!(original.as_ref().state(), SkittleState::Stand);
    }

    #[test]
    fn 立て直しは指定個体の位置と識別子を保ち記録と角度を戻す() {
        let (mut rules, mut skittles, held) = scene(&[[0.0, 0.7]]);
        let mut stick = shot(&held, [0.0, 1.0], 6.0);
        settle(&mut rules, &mut skittles, &mut stick);
        let fallen = skittles[0];
        assert_eq!(fallen.as_ref().state(), SkittleState::Fall);
        let standing = fallen.stand_up();
        rules = create_world(&stick, &[standing]);
        assert_eq!(standing.as_ref().id(), fallen.as_ref().id());
        assert_eq!(standing.as_ref().state(), SkittleState::Stand);
        assert_eq!(standing.position(), fallen.position());
        assert_eq!(standing.angle(), 0.0);
        assert_eq!(rules.hit(&stick, &standing), standing);
        assert_eq!(fallen.as_ref().state(), SkittleState::Fall);
    }

    #[test]
    fn 高速投擲でもすり抜けず同番号の個体を区別して命中状態にする() {
        let (mut rules, mut skittles, held) = scene(&[[0.0, 1.0], [0.2, 1.0]]);
        let original = skittles.to_vec();
        let mut stick = shot(&held, [0.0, 1.0], 120.0);
        for _ in 0..1200 {
            (skittles, stick) = advance(&mut rules, &skittles, stick);
            if skittles
                .iter()
                .any(|p| p.as_ref().state() == SkittleState::Hit)
            {
                break;
            }
            assert_eq!(stick.as_ref().state(), StickState::Move);
        }
        assert_eq!(skittles[0].as_ref().state(), SkittleState::Hit);
        assert_eq!(skittles[1].as_ref().state(), SkittleState::Stand);
        assert_eq!(skittles[0].as_ref().id(), original[0].as_ref().id());
        assert_ne!(original[0].as_ref().id(), original[1].as_ref().id());
        assert!(
            original
                .iter()
                .all(|p| p.as_ref().state() == SkittleState::Stand)
        );
        settle(&mut rules, &mut skittles, &mut stick);
        assert_eq!(skittles[0].as_ref().state(), SkittleState::Fall);
    }

    #[test]
    fn 弱い衝突で動いた個体は停止後に立位へ戻り得点にならない() {
        let (mut rules, mut skittles, held) = scene(&[[0.0, 0.16]]);
        let mut stick = shot(&held, [0.0, 1.0], 0.1);
        let mut touched = false;
        for _ in 0..1200 {
            (skittles, stick) = advance(&mut rules, &skittles, stick);
            touched |= skittles[0].as_ref().state() == SkittleState::Hit;
            if stick.as_ref().state() == StickState::Stop
                && skittles[0].as_ref().state() != SkittleState::Hit
            {
                break;
            }
        }
        assert!(touched);
        assert_eq!(skittles[0].as_ref().state(), SkittleState::Stand);
        assert!(fallen_skittles(&skittles).is_empty());
        assert!(skittles[0].position()[1] > 0.16);
    }

    #[test]
    fn 物理の連鎖は直接の照準外にも届き二本を転倒に数える() {
        let (mut rules, mut skittles, held) = scene(&[[0.02, 0.7], [0.06, 0.85]]);
        let mut stick = shot(&held, [0.0, 1.0], 6.0);
        assert_eq!(rules.aim(&stick, &skittles).len(), 1);
        settle(&mut rules, &mut skittles, &mut stick);
        assert_eq!(fallen_skittles(&skittles).len(), 2);
        assert_ne!(skittles[0].as_ref().id(), skittles[1].as_ref().id());
    }

    #[test]
    fn chain_hitは接触した相手を判定し立て直しは他の個体と棒を変えない() {
        let (mut rules, mut skittles, held) = scene(&[[0.02, 0.7], [0.06, 0.85]]);
        let original = skittles.to_vec();
        let mut stick = shot(&held, [0.0, 1.0], 6.0);
        assert_eq!(rules.chain_hit(&original[0], &original[1]), original[1]);
        settle(&mut rules, &mut skittles, &mut stick);
        let first = skittles[0];
        let second = rules.chain_hit(&first, &original[1]);
        assert_eq!(second.as_ref().state(), SkittleState::Fall);
        let unrelated = GuiSkittle::new(Skittle::new(12), [0.0; 2]);
        assert_eq!(rules.chain_hit(&unrelated, &original[1]), original[1]);
        assert_eq!(rules.chain_hit(&first, &first), first);
        let restored = second.stand_up();
        rules = create_world(&stick, &[first, restored]);
        assert_eq!(restored.position(), second.position());
        assert_eq!(restored.angle(), 0.0);
        assert_eq!(restored.as_ref().state(), SkittleState::Stand);
        assert_eq!(rules.hit(&stick, &restored), restored);
        let unchanged = rules.hit(&stick, &first);
        assert_eq!(*unchanged.as_ref(), *first.as_ref());
        assert_eq!(unchanged.position(), first.position());
        assert_eq!(unchanged.aimed(), first.aimed());
        // 角度→回転→角度の再構築によるf32の丸めだけを許容する。
        assert!((unchanged.angle() - first.angle()).abs() <= f32::EPSILON);
        assert_eq!(second.as_ref().state(), SkittleState::Fall);
    }

    #[test]
    fn 次投でフィールドと個体の位置を保ち前回の転倒を持ち越さない() {
        let (mut rules, mut skittles, held) = scene(&[[0.0, 0.7], [0.04, 0.9]]);
        let mut stick = shot(&held, [0.0, 1.0], 6.0);
        settle(&mut rules, &mut skittles, &mut stick);
        assert!(!fallen_skittles(&skittles).is_empty());
        let positions: Vec<_> = skittles.iter().map(|p| p.position()).collect();
        let ids: Vec<_> = skittles.iter().map(|p| p.as_ref().id()).collect();
        skittles = skittles.iter().copied().map(StandUp::stand_up).collect();
        assert_eq!(
            skittles.iter().map(|p| p.position()).collect::<Vec<_>>(),
            positions
        );
        rules = create_world(&stick, &skittles);
        stick = shot(&stick, [1.0, 0.0], 0.1);
        settle(&mut rules, &mut skittles, &mut stick);
        assert!(fallen_skittles(&skittles).is_empty());
        assert_eq!(
            skittles.iter().map(|p| p.as_ref().id()).collect::<Vec<_>>(),
            ids
        );
    }
}

mod field_boundaries {
    use super::*;

    fn assert_bounces(target: [f32; 2], speed: f32) {
        let (mut rules, mut skittles, held) = scene(&[]);
        let mut stick = shot(&held, target, speed);
        let mut previous = [0.0; 2];
        let mut reflected = [false; 2];
        for _ in 0..1200 {
            (skittles, stick) = advance(&mut rules, &skittles, stick);
            let position = stick.position();
            let angle = stick.angle();
            let half_segment = GuiStick::LENGTH / 2.0 - GuiStick::RADIUS;
            let extents =
                [angle.sin().abs(), angle.cos().abs()].map(|c| GuiStick::RADIUS + half_segment * c);
            for (axis, min, max) in [(0, -0.3, 0.3), (1, -0.15, 1.35)] {
                assert!(
                    position[axis] - extents[axis] >= min - 0.005
                        && position[axis] + extents[axis] <= max + 0.005,
                    "{position:?}, {angle}"
                );
                if (position[axis] - previous[axis]) * target[axis] < -0.00001 {
                    reflected[axis] = true;
                }
            }
            previous = position;
            if stick.as_ref().state() == StickState::Stop {
                break;
            }
        }
        for axis in 0..2 {
            if target[axis] != 0.0 {
                assert!(reflected[axis], "{target:?}");
            }
        }
        assert_eq!(stick.as_ref().state(), StickState::Stop);
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
        let (mut rules, mut skittles, held) = scene(&[[0.0, 0.7]]);
        let mut stick = shot(&held, [0.0, 1.0], 120.0);
        for _ in 0..1200 {
            (skittles, stick) = advance(&mut rules, &skittles, stick);
            let [x, y] = skittles[0].position();
            let radius = GuiSkittle::RADIUS;
            assert!(
                x - radius >= -0.305
                    && x + radius <= 0.305
                    && y - radius >= -0.155
                    && y + radius <= 1.355
            );
            if stick.as_ref().state() == StickState::Stop
                && skittles[0].as_ref().state() != SkittleState::Hit
            {
                break;
            }
        }
        assert_eq!(stick.as_ref().state(), StickState::Stop);
        assert_eq!(fallen_skittles(&skittles).len(), 1);
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
        let disqualified = crate::ScoreBoard::new((0..3).map(|_| rules.calc_score(&[])).collect());
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
