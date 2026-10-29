use rust_molkky::cui::output;
use rust_molkky::{Molkky, Name, ScoreBoard, SimpleMolkky, Skittle, Team};
use std::io::{self, Write};

fn team(name: &str, points: &[i32]) -> Team {
    let history = points
        .iter()
        .map(|&number| {
            if number == 0 {
                SimpleMolkky.calc_score(&[])
            } else {
                SimpleMolkky.calc_score(&[Skittle::new(number)])
            }
        })
        .collect();
    Team::new(Name::new(name)).with_score_board(ScoreBoard::new(history))
}

mod scoreboard {
    use super::*;

    #[test]
    fn 名前と得点と投数と手番を表示する() {
        let teams = [team("赤", &[5]).with_turn(true), team("青", &[])];
        let mut buffer = Vec::new();
        output::scoreboard(&mut buffer, &teams).unwrap();
        let text = String::from_utf8(buffer).unwrap();
        assert!(text.contains("▶ 1. 赤: 5点 (1投)"));
        assert!(text.contains("2. 青: 0点 (0投)"));
    }

    #[test]
    fn 補正と失格と勝利を状態から表示する() {
        let teams = [
            team("赤", &[12, 12, 12, 12, 3]),
            team("青", &[5, 0, 0, 0]),
            team("緑", &[10; 5]),
        ];
        let mut buffer = Vec::new();
        output::scoreboard(&mut buffer, &teams).unwrap();
        let text = String::from_utf8(buffer).unwrap();
        assert!(text.contains("赤: 25点 (5投) [25点に補正]"));
        assert!(text.contains("青: 0点 (4投) [失格]"));
        assert!(text.contains("緑: 50点 (5投) [勝利]"));
    }
}

mod messages {
    use super::*;

    #[test]
    fn 勝者の名前と得点を表示する() {
        let winner = team("赤", &[10; 5]);
        let mut buffer = Vec::new();
        output::finish(&mut buffer, Some(&winner)).unwrap();
        assert!(
            String::from_utf8(buffer)
                .unwrap()
                .contains("赤の勝利です（50点）")
        );
    }

    #[test]
    fn 全失格は勝者なしとして表示する() {
        let mut buffer = Vec::new();
        output::finish(&mut buffer, None).unwrap();
        assert!(
            String::from_utf8(buffer)
                .unwrap()
                .contains("全チームが失格")
        );
    }
}

mod io_contract {
    use super::*;

    #[derive(Default)]
    struct PromptWriter {
        buffer: Vec<u8>,
        flushed: bool,
    }
    impl Write for PromptWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.buffer.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            self.flushed = true;
            Ok(())
        }
    }

    #[test]
    fn 入力待ちの前にプロンプトをflushする() {
        let mut writer = PromptWriter::default();
        output::team_count_prompt(&mut writer).unwrap();
        assert!(writer.flushed);
        assert!(
            String::from_utf8(writer.buffer)
                .unwrap()
                .contains("チーム数")
        );
    }

    struct FailedWriter;
    impl Write for FailedWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn 描画のioエラーを呼び出し側へ返す() {
        assert_eq!(
            output::quit(&mut FailedWriter).unwrap_err().kind(),
            io::ErrorKind::BrokenPipe
        );
    }
}
