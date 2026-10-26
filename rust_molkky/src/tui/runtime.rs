use super::{
    molkky::{MIN_HEIGHT, MIN_WIDTH, MolkkyTui},
    team::TeamTui,
};

use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
};
use rust_molkky::core::{
    Molkky, Position, Stick, Team, calc_score, relocate_skittles, throw_stick,
};

pub struct Runtime {
    names: Vec<String>,
    game: Molkky,
    active: usize,
    message: String,
}

impl Runtime {
    pub fn new(names: Vec<String>) -> Self {
        let teams = names.iter().map(|name| Team::new(name)).collect();
        Self {
            names,
            game: Molkky::new(teams),
            active: 0,
            message: "Space / Enterで最初の投てき。矢印キーで投てき位置を移動できます。".into(),
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
                KeyCode::Enter | KeyCode::Char(' ') => self.throw(),
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
        if self.game.is_finished() {
            return;
        }
        let from = self.game.stick().position();
        if let (Some(x), Some(y)) = (from.x().checked_add(dx), from.y().checked_add(dy)) {
            self.game = self.game.with_stick(Stick::new(Position::new(x, y)));
        }
    }

    fn throw(&mut self) {
        // 終了後はFieldも動かさないよう、乱数生成より先に確認する。
        if self.game.is_finished() {
            return;
        }
        let turn = self
            .game
            .teams()
            .iter()
            .map(|team| team.score_board().history().len())
            .sum::<usize>()
            + 1;
        let before = self.game.teams()[self.active].score_board().value();
        let mut rng = rand::rng();
        let knocked_over = throw_stick(self.game.stick(), self.game.field().skittles(), &mut rng);
        let point = calc_score(&knocked_over);
        let updated = self.game.record_point(self.active, point);
        let field = relocate_skittles(updated.field(), &mut rng);
        self.game = updated.with_field(field);
        self.message =
            TeamTui::new(&self.game.teams()[self.active]).throw_message(turn, before, &point);
        if !self.game.is_finished() {
            // 失格チームを飛ばす。継続中なので必ず投てき可能なチームが存在する。
            self.active = (1..=self.game.teams().len())
                .map(|offset| (self.active + offset) % self.game.teams().len())
                .find(|&index| !self.game.teams()[index].score_board().is_disqualified())
                .expect("投てき可能なチームが存在する");
        }
    }

    fn render(&self, frame: &mut Frame) {
        frame.render_widget(
            MolkkyTui::new(&self.game, self.active, &self.message),
            frame.area(),
        );
    }
}

#[cfg(test)]
#[path = "../../tests/tui/runtime.rs"]
mod tests;
