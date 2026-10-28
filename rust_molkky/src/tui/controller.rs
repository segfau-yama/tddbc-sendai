use crate::{
    game::{compose_game, relocate_skittles, throw_message},
    view::{
        field::FieldLayout,
        molkky::{MIN_HEIGHT, MIN_WIDTH},
    },
};
use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
};
use rust_molkky::core::{
    Field, GameStatus, Stick, Team, calc_score, judge_game, next_team, record_point, throw_stick,
};

pub struct Controller {
    names: Vec<String>,
    teams: Vec<Team>,
    field: Field,
    stick: Stick,
    layout: FieldLayout,
    active: usize,
    message: String,
}

impl Controller {
    pub fn new(names: Vec<String>) -> Self {
        let teams = names.iter().map(|name| Team::new(name)).collect();
        Self {
            names,
            teams,
            field: Field::default(),
            stick: Stick::default(),
            layout: FieldLayout::default(),
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
        if matches!(judge_game(&self.teams), GameStatus::Finished { .. }) {
            return;
        }
        if let Some(position) = self.layout.stick.translated(dx, dy) {
            self.layout.stick = position;
        }
    }

    fn throw_with_rng(&mut self, rng: &mut impl rand::Rng) {
        // 終了後はFieldも動かさないよう、乱数生成より先に確認する。
        if matches!(judge_game(&self.teams), GameStatus::Finished { .. }) {
            return;
        }
        let turn = self
            .teams
            .iter()
            .map(|team| team.score_board().history().len())
            .sum::<usize>()
            + 1;
        let before = self.teams[self.active].score_board().value();
        let knocked_over = throw_stick(&self.stick, self.field.skittles(), rng);
        let point = calc_score(&knocked_over);
        let updated = record_point(&self.teams, self.active, point);
        self.layout = relocate_skittles(&self.layout, &knocked_over, rng);
        self.teams = updated;
        self.message = throw_message(&self.teams[self.active], turn, before, point);
        if let Some(next) = next_team(&self.teams, self.active) {
            self.active = next;
        }
    }

    fn render(&self, frame: &mut Frame) {
        frame.render_widget(
            compose_game(
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
