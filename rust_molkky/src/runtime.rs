use std::{collections::BTreeMap, io};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Cell, Paragraph, Row, Table, TableState, Wrap,
        canvas::{Canvas, Line as CanvasLine},
    },
};
use rust_molkky::{Field, Molkky, Position, Team, random_throw};

const MIN_WIDTH: u16 = 72;
const MIN_HEIGHT: u16 = 24;

pub struct Runtime {
    names: Vec<String>,
    game: Molkky,
    field: Field,
    from: Position,
    active: usize,
    message: String,
}

impl Runtime {
    pub fn new(names: Vec<String>) -> Self {
        let teams = names.iter().map(|name| Team::new(name)).collect();
        Self {
            names,
            game: Molkky::new(teams),
            field: Field::default(),
            from: Position::new(0, 0),
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
        if let (Some(x), Some(y)) = (self.from.x().checked_add(dx), self.from.y().checked_add(dy)) {
            self.from = Position::new(x, y);
        }
    }

    fn throw(&mut self) {
        // 終了後はFieldも動かさないよう、乱数生成より先に確認する。
        if self.game.result().is_some() {
            return;
        }
        let turn = self
            .game
            .teams()
            .iter()
            .map(|team| team.score().history().len())
            .sum::<usize>()
            + 1;
        let before = self.game.teams()[self.active].score().value();
        let thrown = random_throw(&mut self.field, self.from, &mut rand::rng());
        let points = self.game.record_throw(self.active, &thrown);
        let team = &self.game.teams()[self.active];
        let score = team.score().value();
        let reset = if before + points > 50 {
            "  50点を超過したため25点へ"
        } else {
            ""
        };
        self.message = format!(
            "第{turn}投  {}: {points}点  得点 {before} → {score}{reset}",
            team.name()
        );
        if self.game.result().is_none() {
            self.active = (self.active + 1) % self.game.teams().len();
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
        self.render_board(frame, board);
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
                    "投てき位置: ({}, {})   50点ぴったりで勝利 / 超過すると25点",
                    self.from.x(),
                    self.from.y()
                )),
            ]),
            help,
        );
    }

    fn render_scores(&self, frame: &mut Frame, area: Rect) {
        let winner = self.game.result();
        let rows = self.game.teams().iter().enumerate().map(|(index, team)| {
            let history = team.score().history();
            let recent = history[history.len().saturating_sub(6)..]
                .iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join(" ");
            let marker = if winner == Some(index) {
                "勝"
            } else if index == self.active {
                "▶"
            } else {
                ""
            };
            Row::new([
                Cell::from(marker),
                Cell::from(team.name()),
                Cell::from(format!("{} / 50", team.score().value())),
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
                Constraint::Length(3),
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

    fn render_board(&self, frame: &mut Frame, area: Rect) {
        let from_x = f64::from(self.from.x());
        let from_y = f64::from(self.from.y());
        let (mut left, mut right, mut bottom, mut top) = (from_x, from_x, from_y, from_y);
        let mut labels: BTreeMap<(i32, i32), Vec<String>> = BTreeMap::new();
        for pin in self.field.skittles() {
            let position = pin.position();
            let x = f64::from(position.x());
            let y = f64::from(position.y());
            left = left.min(x);
            right = right.max(x);
            bottom = bottom.min(y);
            top = top.max(y);
            labels
                .entry((position.x(), position.y()))
                .or_default()
                .push(format!("{:02}", pin.number()));
        }
        let canvas = Canvas::default()
            .block(Block::bordered().title("盤面  番号=ピン  @=投てき位置"))
            .x_bounds([left - 3.0, right + 3.0])
            .y_bounds([bottom - 2.0, top + 2.0])
            .paint(|context| {
                context.draw(&CanvasLine {
                    x1: left - 3.0,
                    y1: from_y,
                    x2: right + 3.0,
                    y2: from_y,
                    color: Color::DarkGray,
                });
                context.draw(&CanvasLine {
                    x1: from_x,
                    y1: bottom - 2.0,
                    x2: from_x,
                    y2: top + 2.0,
                    color: Color::DarkGray,
                });
                for (&(x, y), numbers) in &labels {
                    context.print(
                        f64::from(x),
                        f64::from(y),
                        Line::from(Span::styled(
                            numbers.join("/"),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )),
                    );
                }
                context.print(
                    from_x,
                    from_y,
                    Line::from(Span::styled(
                        "@",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )),
                );
            });
        frame.render_widget(canvas, area);
    }

    fn render_positions(&self, frame: &mut Frame, area: Rect) {
        let rows = self.field.skittles().iter().map(|pin| {
            let position = pin.position();
            Row::new([
                format!("{:02}", pin.number()),
                format!("({:>3},{:>3})", position.x(), position.y()),
                format!("{:>5.1}%", pin.fall_probability(self.from) * 100.0),
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
