use crate::Id;

pub struct CuiId(Id);

impl From<Id> for CuiId {
    fn from(value: Id) -> Self {
        Self(value)
    }
}

impl AsRef<Id> for CuiId {
    fn as_ref(&self) -> &Id {
        &self.0
    }
}
