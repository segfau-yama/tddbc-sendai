#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    Won { team_index: usize },
    FinishedWithoutWinner,
}
