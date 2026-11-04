use crate::core::{Field, Id, Skittle};

/// 全チームが共有するフィールドと個体集合。現在位置は物理側で扱う。
#[derive(Clone)]
pub struct GuiField {
    field: Field,
    skittles: Vec<Skittle>,
}

impl Default for GuiField {
    fn default() -> Self {
        Self::new(Field::new(), (1..=12).map(Skittle::new).collect())
    }
}

impl GuiField {
    pub const BOUNDS: [f32; 4] = [-0.3, -0.15, 0.6, 1.5];
    pub const WALL_THICKNESS: f32 = 0.02;
    pub const THROW_POSITION: [f32; 2] = [0.0, 0.0];

    pub fn new(field: Field, skittles: Vec<Skittle>) -> Self {
        Self { field, skittles }
    }

    pub fn field(&self) -> &Field {
        &self.field
    }

    pub fn skittles(&self) -> &[Skittle] {
        &self.skittles
    }

    pub fn initial_positions(&self, radius: f32) -> Vec<(Id, [f32; 2])> {
        let rows: &[&[i32]] = &[&[1, 2], &[3, 10, 4], &[5, 11, 12, 6], &[7, 9, 8]];
        rows.iter()
            .enumerate()
            .flat_map(|(row, numbers)| {
                numbers.iter().enumerate().map(move |(column, number)| {
                    let skittle = self
                        .skittles
                        .iter()
                        .find(|skittle| skittle.number() == *number)
                        .unwrap();
                    let x = (2.0 * column as f32 - (numbers.len() - 1) as f32) * radius;
                    (
                        skittle.id(),
                        [x, 1.0 + row as f32 * 3.0_f32.sqrt() * radius],
                    )
                })
            })
            .collect()
    }
}

impl AsRef<Field> for GuiField {
    fn as_ref(&self) -> &Field {
        &self.field
    }
}
