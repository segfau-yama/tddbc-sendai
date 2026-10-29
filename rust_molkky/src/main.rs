use std::{io, process::ExitCode};

fn main() -> ExitCode {
    let names = std::env::args().skip(1).collect::<Vec<_>>();
    let result =
        rust_molkky::cui::main::run(&mut io::stdin().lock(), &mut io::stdout().lock(), &names);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("エラー: {error}");
            ExitCode::FAILURE
        }
    }
}
