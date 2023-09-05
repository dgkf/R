/// Describes how a type can be coerced into a logical representation
pub trait IntoLogical {
    type Output;
    fn as_logical(self) -> Self::Output;
}