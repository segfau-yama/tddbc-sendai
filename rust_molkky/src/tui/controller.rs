use crate::{
    game::{FieldLayout, compose_game, relocate_skittles, throw_message, winning_team},
    view::molkky::{MIN_HEIGHT, MIN_WIDTH},
};
use rust_molkky::Skittles;
use rust_molkky::simulation::{ArrayField, RandomStick, StandardFieldGenerator};
use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
};
use rust_molkky::core::{Molkky, Name, ScoreState, SimpleMolkky, SkittleState, Team, Teams};

pub struct Controller {
    names: Vec<Name>,
    rules: SimpleMolkky,
    teams: Vec<Team>,
    throw_counts: Vec<usize>,
    field: ArrayField,
    stick: RandomStick,
    layout: FieldLayout,
    message: String,
}

impl Controller {
    pub fn new(names: Vec<Name>) -> Self {
        let throw_counts = vec![0; names.len()];
        let teams: Vec<_> = names.iter().cloned().map(Team::new).collect();
        let teams = Teams::new(&teams).next_team();
        let field = StandardFieldGenerator.generate();
        let layout = FieldLayout::new(field.skittles());
        Self {
            names,
            rules: SimpleMolkky,
            teams,
            throw_counts,
            field,
            stick: RandomStick::default(),
            layout,
            message: "Space / Enterで最初の投てき。矢印キーで棒の表示位置を移動できます（確率は変わりません）。".into(),
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('q' | 'Q'))
                || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
            {
                return Ok(());
            }
            let size = terminal.size()?;
            if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
                continue;
            }
            match key.code {
                KeyCode::Enter | KeyCode::Char(' ') => self.throw_with_rng(&mut rand::rng()),
                KeyCode::Left => self.move_from(-1, 0),
                KeyCode::Right => self.move_from(1, 0),
                KeyCode::Up => self.move_from(0, 1),
                KeyCode::Down => self.move_from(0, -1),
                KeyCode::Char('r' | 'R') => self = Self::new(self.names),
                _ => {}
            }
        }
    }

    fn move_from(&mut self, dx: i32, dy: i32) {
        if winning_team(&self.teams).is_some() || !self.teams.iter().any(Team::turn) {
            return;
        }
        if let Some(position) = self.layout.stick.translated(dx, dy) {
            self.layout.stick = position;
        }
    }

    fn throw_with_rng(&mut self, rng: &mut impl rand::Rng) {
        // 終了後はFieldも動かさないよう、乱数生成より先に確認する。
        if winning_team(&self.teams).is_some() {
            return;
        }
        let Some(active) = self.teams.iter().position(Team::turn) else {
            return;
        };
        if self.teams[active].score_board().state() == ScoreState::Disqualified {
            return;
        }
        let before = self.teams[active].score_board().total();
        let updated = self.rules.hit(
            self.stick.stick(),
            Skittles::new(self.field.skittles()),
            &mut |_, pins| {
                let pins = pins.as_slice();
                self.stick.judge(pins, rng)
            },
        );
        let knocked_over: Vec<_> = updated
            .iter()
            .copied()
            .filter(|pin| {
                pin.state() == SkittleState::Fall
                    && self.field.skittles().iter().any(|before| {
                        before.id() == pin.id() && before.state() == SkittleState::Stand
                    })
            })
            .collect();
        let point = Skittles::new(&knocked_over).calc_score();
        let board = self.teams[active].score_board().add(point);
        self.teams[active] = self.teams[active].with_score_board(board);
        self.throw_counts[active] += 1;
        let turn = self.throw_counts.iter().sum();
        self.layout = relocate_skittles(&self.layout, &knocked_over, rng);
        // 次の投てきに備え、移動先で立て直す。個体とFieldのIDは保つ。
        self.field = ArrayField::new(
            *self.field.field(),
            updated
                .iter()
                .map(|pin| pin.with_state(SkittleState::Stand))
                .collect(),
        );
        self.message = throw_message(&self.teams[active], turn, before, point);
        self.teams = Teams::new(&self.teams).next_team();
    }

    fn render(&self, frame: &mut Frame) {
        frame.render_widget(
            compose_game(
                &self.teams,
                &self.throw_counts,
                &self.field,
                &self.stick,
                &self.layout,
                &self.message,
            ),
            frame.area(),
        );
    }
}

#[cfg(test)]
#[path = "../../tests/tui/controller.rs"]
mod tests;
