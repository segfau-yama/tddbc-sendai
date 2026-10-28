#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    /// 勝者のインデックス。全チーム失格なら勝者はいない。
    Finished {
        winner: Option<usize>,
    },
}
