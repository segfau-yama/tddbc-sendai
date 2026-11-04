use crate::Name;

pub struct CuiName(Name);

impl From<Name> for CuiName {
    fn from(value: Name) -> Self {
        Self(value)
    }
}

impl AsRef<Name> for CuiName {
    fn as_ref(&self) -> &Name {
        &self.0
    }
}
