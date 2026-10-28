use crate::{
    game::{FieldLayout, compose_game, relocate_skittles, throw_message, winning_team},
    view::molkky::{MIN_HEIGHT, MIN_WIDTH},
};
use rust_molkky::Stick;
use rust_molkky::simulation::{ArrayField, StandardFieldGenerator};
use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
};
use rust_molkky::core::{Field, FieldGenerator, Molkky, Name, ScoreState, SimpleMolkky, Team};

pub struct Controller {
    names: Vec<Name>,
    rules: SimpleMolkky,
    teams: Vec<Team>,
    field: ArrayField,
    stick: Stick,
    layout: FieldLayout,
    message: String,
}

impl Controller {
    pub fn new(names: Vec<Name>) -> Self {
        let teams = names
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, name)| Team::new(name).with_turn(i == 0))
            .collect();
        let field = StandardFieldGenerator.generate();
        let layout = FieldLayout::new(field.skittles());
        Self {
            names,
            rules: SimpleMolkky,
            teams,
            field,
            stick: Stick::default(),
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
        if winning_team(&self.rules, &self.teams).is_some()
            || self.rules.next_team(&self.teams).is_none()
        {
            return;
        }
        if let Some(position) = self.layout.stick.translated(dx, dy) {
            self.layout.stick = position;
        }
    }

    fn throw_with_rng(&mut self, rng: &mut impl rand::Rng) {
        // 終了後はFieldも動かさないよう、乱数生成より先に確認する。
        if winning_team(&self.rules, &self.teams).is_some() {
            return;
        }
        let Some(current) = self.rules.next_team(&self.teams) else {
            return;
        };
        if current.score_board().state() == ScoreState::Disqualified {
            return;
        }
        let active = self
            .teams
            .iter()
            .position(|team| team.id() == current.id())
            .unwrap();
        let turn = self
            .teams
            .iter()
            .map(|team| team.score_board().history().len())
            .sum::<usize>()
            + 1;
        let before = self.teams[active].score_board().total();
        let knocked_over = self.rules.throw(&self.stick, self.field.skittles(), rng);
        let point = self.rules.calc_score(&knocked_over);
        let board = self.teams[active].score_board().add(point);
        self.teams[active] = self.teams[active].with_score_board(board);
        self.layout = relocate_skittles(&self.layout, &knocked_over, rng);
        self.message = throw_message(&self.teams[active], turn, before, point);
        let next = if winning_team(&self.rules, &self.teams).is_some() {
            None
        } else {
            (1..=self.teams.len())
                .map(|offset| (active + offset) % self.teams.len())
                .find(|&index| self.teams[index].score_board().state() != ScoreState::Disqualified)
        };
        self.teams = self
            .teams
            .iter()
            .enumerate()
            .map(|(i, team)| team.with_turn(next == Some(i)))
            .collect();
    }

    fn render(&self, frame: &mut Frame) {
        frame.render_widget(
            compose_game(
                &self.rules,
                &self.teams,
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
