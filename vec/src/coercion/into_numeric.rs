pub trait IntoNumeric {
    type Output;
    fn as_numeric(self) -> Self::Output;
}