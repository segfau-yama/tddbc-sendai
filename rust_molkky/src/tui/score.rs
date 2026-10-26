use rust_molkky::core::{CONSECUTIVE_ZERO_LIMIT, RESET_SCORE, ScoreBoard, TARGET_SCORE};

use super::point::PointTui;

pub(super) struct ScoreBoardTui<'a> {
    board: &'a ScoreBoard,
}

impl<'a> ScoreBoardTui<'a> {
    pub(super) fn new(board: &'a ScoreBoard) -> Self {
        Self { board }
    }

    pub(super) fn value_label(&self) -> String {
        format!("{} / {TARGET_SCORE}", self.board.value())
    }

    pub(super) fn throw_count_label(&self) -> String {
        self.board.history().len().to_string()
    }

    pub(super) fn recent_points_label(&self) -> String {
        let history = self.board.history();
        history[history.len().saturating_sub(6)..]
            .iter()
            .map(|point| PointTui::new(point).label())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub(super) fn change_notice(&self) -> String {
        let reset = if self.board.was_reset() {
            format!("  {TARGET_SCORE}点を超過したため{RESET_SCORE}点へ")
        } else {
            String::new()
        };
        let disqualification = if self.board.is_disqualified() {
            format!("  {CONSECUTIVE_ZERO_LIMIT}回連続0点のため失格")
        } else {
            String::new()
        };
        format!("{reset}{disqualification}")
    }
}

#[cfg(test)]
#[path = "../../tests/tui/score.rs"]
mod tests;
