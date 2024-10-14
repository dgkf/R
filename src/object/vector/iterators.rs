use std::ops::Deref;

use super::coercion::{CoercibleInto, CommonNum, MinimallyNumeric};

/// Map an iterator of pairs into a pair of common numeric types
///
/// Accept an iterator of pairs of numeric (or numeric-coercible) values and
/// returns an iterator of pairs of numerics coerced to the least greater common
/// type, assuming a hierarchy of representations.
///
/// # Arguments
///
/// * `i` - An iterator over pairs of numeric (or numeric-coercible) values
///
pub fn map_common_numeric<I, LItem, RItem, DLItem, DRItem, LNum, RNum, Output>(
    i: I,
) -> impl Iterator<Item = (Output, Output)>
where
    // iterator over pairs of items
    I: IntoIterator<Item = (LItem, RItem)>,
    // and item should be coercible to numeric
    LItem: MinimallyNumeric<As = LNum> + Deref<Target = DLItem>,
    RItem: MinimallyNumeric<As = RNum> + Deref<Target = DRItem>,
    DLItem: CoercibleInto<LNum> + Clone,
    DRItem: CoercibleInto<RNum> + Clone,
    // and those numerics should be coercible to a common numeric
    (LNum, RNum): CommonNum<Common = Output>,
{
    i.into_iter().map(|(l, r)| {
        (
            CoercibleInto::<LNum>::coerce_into(l.clone()),
            CoercibleInto::<RNum>::coerce_into(r.clone()),
        )
            .into_common()
    })
}
