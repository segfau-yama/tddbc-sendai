mod game;
mod tui;
mod view;

use rust_molkky::Name;
use std::io::{self, IsTerminal};

fn main() -> io::Result<()> {
    let mut names: Vec<String> = std::env::args().skip(1).collect();
    if names.iter().any(|name| name == "--help" || name == "-h") {
        println!(
            "モルック TUI\n\n起動: cargo run -- [チーム名 ...]\n\nSpace / Enter: 投てき\n矢印キー: 棒の表示位置を移動（確率は変わりません）\nR: 新しいゲーム\nQ / Esc / Ctrl+C: 終了"
        );
        return Ok(());
    }
    if names.is_empty() {
        names = vec!["赤チーム".into(), "青チーム".into()];
    }
    let names: Vec<Name> = names
        .iter()
        .map(|name| Name::new(name))
        .collect::<Result<_, _>>()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?;
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(io::Error::other(
            "対話型ターミナルでcargo runを実行してください",
        ));
    }
    ratatui::run(|terminal| tui::Controller::new(names).run(terminal))
}

#[cfg(test)]
#[path = "../tests/support/mod.rs"]
mod test_support;
