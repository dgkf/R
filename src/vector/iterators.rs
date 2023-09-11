use std::ops::Deref;
use crate::vector::coercion::*;

/// Zip iterators into recycling vectors, extending to longest length
///
/// This operation does not do any vector length matching, elements will be
/// recycled even if they do not repeat an even number of times.
///
/// ```rust
/// use r::vector::iterators::zip_recycle;
///
/// let x = vec![1, 2, 3, 4];
/// let y = vec![2, 4];
/// let z: Vec<_> = zip_recycle(x, y).collect();
/// ````
///
pub fn zip_recycle<'a, L, R, LItem, RItem>(
    l: L,
    r: R,
) -> impl Iterator<Item = (LItem, RItem)> + 'a
where
    L: ExactSizeIterator + Iterator<Item = LItem> + Clone + 'a,
    R: ExactSizeIterator + Iterator<Item = RItem> + Clone + 'a,
    LItem: Clone + 'a,
    RItem: Clone + 'a,
    // LIter: ExactSizeIterator + Iterator<Item = LItem> + Clone + 'a,
    // RIter: ExactSizeIterator + Iterator<Item = RItem> + Clone + 'a,
{
    let l = l.into_iter();
    let r = r.into_iter();
    let n = std::cmp::max(l.len(), r.len());
    l.cycle().zip(r.cycle()).take(n)
}

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
    i.into_iter()
        .map(|(l, r)| (
            CoercibleInto::<LNum>::coerce_into(l.clone()),
            CoercibleInto::<RNum>::coerce_into(r.clone())
        ).as_common())
}