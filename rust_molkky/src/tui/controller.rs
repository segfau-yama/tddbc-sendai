use crate::{
    game::{FieldLayout, compose_game, relocate_skittles, throw_message},
    view::molkky::{MIN_HEIGHT, MIN_WIDTH},
};
use rust_molkky::simulation::{ArrayField, RandomThrow, StandardFieldGenerator, Stick};
use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
};
use rust_molkky::core::{
    Field, FieldGenerator, GameStatus, Molkky, Name, SimpleMolkky, Team, throw_stick,
};

pub struct Controller {
    names: Vec<Name>,
    rules: SimpleMolkky,
    teams: Vec<Team>,
    field: ArrayField,
    stick: Stick,
    layout: FieldLayout,
    active: usize,
    message: String,
}

impl Controller {
    pub fn new(names: Vec<Name>) -> Self {
        let teams = names.iter().cloned().map(Team::new).collect();
        let field = StandardFieldGenerator.generate();
        let layout = FieldLayout::new(field.skittles());
        Self {
            names,
            rules: SimpleMolkky,
            teams,
            field,
            stick: Stick::default(),
            layout,
            active: 0,
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
        if matches!(
            self.rules.judge_game(&self.teams),
            GameStatus::Finished { .. }
        ) {
            return;
        }
        if let Some(position) = self.layout.stick.translated(dx, dy) {
            self.layout.stick = position;
        }
    }

    fn throw_with_rng(&mut self, rng: &mut impl rand::Rng) {
        // 終了後はFieldも動かさないよう、乱数生成より先に確認する。
        if matches!(
            self.rules.judge_game(&self.teams),
            GameStatus::Finished { .. }
        ) {
            return;
        }
        let turn = self
            .teams
            .iter()
            .map(|team| team.score_board().history().len())
            .sum::<usize>()
            + 1;
        let before = self.teams[self.active].score_board().value();
        let outcome = throw_stick(&self.field, &mut RandomThrow::new(&self.stick, rng)).unwrap();
        let (field, knocked_over) = outcome.into_parts();
        let point = self.rules.calc_score(&knocked_over);
        let updated = self.rules.record_point(&self.teams, self.active, point);
        self.layout = relocate_skittles(&self.layout, &knocked_over, rng);
        self.teams = updated;
        self.field = field;
        self.message = throw_message(&self.teams[self.active], turn, before, point);
        if let Some(next) = self.rules.next_team(&self.teams, self.active) {
            self.active = next;
        }
    }

    fn render(&self, frame: &mut Frame) {
        frame.render_widget(
            compose_game(
                &self.rules,
                &self.teams,
                &self.field,
                &self.stick,
                &self.layout,
                self.active,
                &self.message,
            ),
            frame.area(),
        );
    }
}

#[cfg(test)]
#[path = "../../tests/tui/controller.rs"]
mod tests;
