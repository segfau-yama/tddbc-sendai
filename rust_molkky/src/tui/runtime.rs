use super::field::FieldTui;

use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Cell, Paragraph, Row, Table, TableState, Wrap},
};
use rust_molkky::core::{
    CONSECUTIVE_ZERO_LIMIT, Molkky, Position, RESET_SCORE, Stick, TARGET_SCORE, Team, calc_score,
    fall_probability, relocate_skittles, throw_stick,
};

const MIN_WIDTH: u16 = 72;
const MIN_HEIGHT: u16 = 24;

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
        let points = point.value();
        let team = &self.game.teams()[self.active];
        let score = team.score_board().value();
        let reset = if team.score_board().was_reset() {
            format!("  {TARGET_SCORE}点を超過したため{RESET_SCORE}点へ")
        } else {
            String::new()
        };
        let disqualification = if team.score_board().is_disqualified() {
            format!("  {CONSECUTIVE_ZERO_LIMIT}回連続0点のため失格")
        } else {
            String::new()
        };
        self.message = format!(
            "第{turn}投  {}: {points}点  得点 {before} → {score}{reset}{disqualification}",
            team.name()
        );
        if !self.game.is_finished() {
            // 失格チームを飛ばす。継続中なので必ず投てき可能なチームが存在する。
            self.active = (1..=self.game.teams().len())
                .map(|offset| (self.active + offset) % self.game.teams().len())
                .find(|&index| !self.game.teams()[index].score_board().is_disqualified())
                .expect("投てき可能なチームが存在する");
        }
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
            frame.render_widget(Paragraph::new(format!(
                "端末を{MIN_WIDTH}列 × {MIN_HEIGHT}行以上に広げてください。\n現在: {}列 × {}行\nQ / Escで終了",
                area.width, area.height,
            )).block(Block::bordered().title("MÖLKKY")), area);
            return;
        }
        let [heading, scores, field, message, help] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .areas(area);
        let banner = match self.game.result() {
            Some(winner) => format!(
                " MÖLKKY  |  {}の勝利！  Rで新しいゲーム",
                self.game.teams()[winner].name()
            ),
            None if self.game.is_finished() => {
                " MÖLKKY  |  全チームが失格。勝者なし。Rで新しいゲーム".into()
            }
            None => format!(
                " MÖLKKY  |  {}の投てき  |  {}チーム",
                self.game.teams()[self.active].name(),
                self.game.teams().len()
            ),
        };
        frame.render_widget(
            Paragraph::new(banner).style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            heading,
        );
        self.render_scores(frame, scores);
        let [board, positions] =
            Layout::horizontal([Constraint::Percentage(62), Constraint::Percentage(38)])
                .areas(field);
        FieldTui::new(self.game.field(), self.game.stick()).draw(frame, board);
        self.render_positions(frame, positions);
        frame.render_widget(
            Paragraph::new(self.message.as_str()).wrap(Wrap { trim: false }),
            message,
        );
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("Space/Enter", Style::default().fg(Color::Yellow)),
                    Span::raw(" 投てき  "),
                    Span::styled("矢印", Style::default().fg(Color::Yellow)),
                    Span::raw(" 位置移動  "),
                    Span::styled("R", Style::default().fg(Color::Yellow)),
                    Span::raw(" 最初から  "),
                    Span::styled("Q/Esc", Style::default().fg(Color::Yellow)),
                    Span::raw(" 終了"),
                ]),
                Line::from(format!(
                    "投てき位置: ({}, {})   {TARGET_SCORE}点ぴったりで勝利 / 超過すると{RESET_SCORE}点",
                    self.game.stick().position().x(),
                    self.game.stick().position().y()
                )),
            ]),
            help,
        );
    }

    fn render_scores(&self, frame: &mut Frame, area: Rect) {
        let winner = self.game.result();
        let rows = self.game.teams().iter().enumerate().map(|(index, team)| {
            let history = team.score_board().history();
            let recent = history[history.len().saturating_sub(6)..]
                .iter()
                .map(|point| point.value().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let marker = if winner == Some(index) {
                "勝"
            } else if team.score_board().is_disqualified() {
                "失格"
            } else if index == self.active {
                "▶"
            } else {
                ""
            };
            Row::new([
                Cell::from(marker),
                Cell::from(team.name()),
                Cell::from(format!("{} / {TARGET_SCORE}", team.score_board().value())),
                Cell::from(history.len().to_string()),
                Cell::from(recent),
            ])
            .style(if winner == Some(index) {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            })
        });
        let table = Table::new(
            rows,
            [
                Constraint::Length(5),
                Constraint::Min(10),
                Constraint::Length(8),
                Constraint::Length(5),
                Constraint::Min(16),
            ],
        )
        .header(
            Row::new(["", "チーム", "得点", "投数", "直近の点数"])
                .style(Style::default().fg(Color::Cyan)),
        )
        .block(Block::bordered().title("得点表"))
        .row_highlight_style(Style::default().bg(Color::DarkGray));
        frame.render_stateful_widget(
            table,
            area,
            &mut TableState::default().with_selected(Some(self.active)),
        );
    }

    fn render_positions(&self, frame: &mut Frame, area: Rect) {
        let rows = self.game.field().skittles().iter().map(|pin| {
            let position = pin.position();
            Row::new([
                format!("{:02}", pin.number()),
                format!("({:>3},{:>3})", position.x(), position.y()),
                format!("{:>5.1}%", fall_probability(self.game.stick(), pin) * 100.0),
            ])
        });
        frame.render_widget(
            Table::new(
                rows,
                [
                    Constraint::Length(3),
                    Constraint::Min(9),
                    Constraint::Length(6),
                ],
            )
            .block(Block::bordered().title("ピン位置 (x,y) / 確率")),
            area,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};
    use rust_molkky::core::{Field, ScoreBoard, Skittle};

    fn rendered(runtime: &Runtime) -> String {
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        terminal.draw(|frame| runtime.render(frame)).unwrap();
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
            .split_whitespace()
            .collect()
    }

    fn almost_disqualified() -> ScoreBoard {
        let missed = calc_score(&[]);
        ScoreBoard::new().add(missed).add(missed)
    }

    #[test]
    fn 残り一チームの勝利後は履歴と配置と棒を変えない() {
        let mut runtime = Runtime::new(vec!["赤".into(), "青".into()]);
        runtime.game = Molkky::new(vec![
            Team::new("赤").with_score_board(almost_disqualified()),
            Team::new("青"),
        ])
        .with_field(Field::new(vec![]));
        runtime.throw();
        assert!(runtime.game.teams()[0].score_board().is_disqualified());
        assert!(runtime.game.is_finished());
        assert_eq!(runtime.game.result(), Some(1));
        let history = runtime.game.teams()[0].score_board().history().to_vec();
        let pins = runtime.game.field().skittles().to_vec();
        let stick = *runtime.game.stick();
        runtime.throw();
        runtime.move_from(1, 0);
        assert_eq!(runtime.game.teams()[0].score_board().history(), history);
        assert_eq!(runtime.game.field().skittles(), pins);
        assert_eq!(*runtime.game.stick(), stick);
        assert!(rendered(&runtime).contains("青の勝利"));
    }

    #[test]
    fn 失格チームを飛ばして二チーム以上なら継続する() {
        let mut runtime = Runtime::new(vec!["赤".into(), "青".into(), "緑".into()]);
        runtime.game = Molkky::new(vec![
            Team::new("赤").with_score_board(almost_disqualified()),
            Team::new("青"),
            Team::new("緑"),
        ])
        .with_field(Field::new(vec![]));
        runtime.throw();
        assert!(!runtime.game.is_finished());
        assert_eq!(runtime.active, 1);
        runtime.throw();
        assert_eq!(runtime.active, 2);
        runtime.throw();
        assert_eq!(runtime.active, 1);
        assert_eq!(runtime.game.teams()[0].score_board().history().len(), 3);
        assert_eq!(runtime.game.teams()[1].score_board().history().len(), 1);
        assert_eq!(runtime.game.teams()[2].score_board().history().len(), 1);
    }

    #[test]
    fn 一チーム開始なら三回連続0点で勝者なし終了する() {
        let mut runtime = Runtime::new(vec!["赤".into()]);
        runtime.game = runtime.game.with_field(Field::new(vec![]));
        assert!(!runtime.game.is_finished());
        for _ in 0..3 {
            runtime.throw();
        }
        assert!(runtime.game.is_finished());
        assert_eq!(runtime.game.result(), None);
        assert_eq!(runtime.game.teams()[0].score_board().history().len(), 3);
        runtime.throw();
        assert_eq!(runtime.game.teams()[0].score_board().history().len(), 3);
        assert!(rendered(&runtime).contains("全チームが失格"));
    }

    #[test]
    fn 次のチームも直前の投てきで移動した共通フィールドと棒を使う() {
        let mut runtime = Runtime::new(vec!["赤".into(), "青".into()]);
        runtime.game = runtime.game.with_field(Field::new(vec![Skittle::new(5)]));
        runtime.throw();
        assert_eq!(runtime.game.teams()[0].score_board().value(), 5);
        assert_eq!(runtime.active, 1);
        let moved = runtime.game.field().skittles()[0].position();
        assert_ne!(moved, Position::new(0, 0));
        runtime.game = runtime.game.with_stick(Stick::new(moved));
        runtime.throw();
        assert_eq!(runtime.game.teams()[1].score_board().value(), 5);
        assert_ne!(runtime.game.field().skittles()[0].position(), moved);
        assert_eq!(runtime.game.stick().position(), moved);
        assert_eq!(runtime.game.teams()[0].score_board().history().len(), 1);
        assert_eq!(runtime.game.teams()[1].score_board().history().len(), 1);
    }
}
