use std::{io, process::ExitCode};

fn main() -> ExitCode {
    let names = std::env::args().skip(1).collect::<Vec<_>>();
    let stick = rust_molkky::cui::models::CuiStick::default();
    let result = rust_molkky::cui::run::run(
        &mut io::stdin().lock(),
        &mut io::stdout().lock(),
        &names,
        &stick,
    );
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("エラー: {error}");
            ExitCode::FAILURE
        }
    }
}
