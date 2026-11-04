use super::GuiMolkky;
use crate::{
    Field, Skittle,
    gui_2d::models::{GuiField, GuiSkittle, GuiStick},
};

/// 全チーム共通の標準配置、棒、対応する物理ワールドを組み立てる。
pub fn create_field() -> (GuiField, GuiStick, GuiMolkky) {
    let field = GuiField::new(Field::new(), (1..=12).map(Skittle::new).collect());
    let stick = GuiStick::default();
    let rows: &[&[i32]] = &[&[1, 2], &[3, 10, 4], &[5, 11, 12, 6], &[7, 9, 8]];
    let radius = GuiSkittle::RADIUS;
    let placements: Vec<_> = rows
        .iter()
        .enumerate()
        .flat_map(|(row, numbers)| {
            let field = &field;
            numbers.iter().enumerate().map(move |(column, number)| {
                let skittle = field
                    .skittles()
                    .iter()
                    .find(|s| s.number() == *number)
                    .unwrap();
                let x = (2.0 * column as f32 - (numbers.len() - 1) as f32) * radius;
                (
                    skittle.id(),
                    [x, 1.0 + row as f32 * 3.0_f32.sqrt() * radius],
                )
            })
        })
        .collect();
    let rules = GuiMolkky::new(&stick, GuiField::THROW_POSITION, &placements);
    (field, stick, rules)
}
