use crate::vector::coercion::*;

/// Zip iterators into recycling vectors, extending to longest length
///
/// This operation does not do any vector length matching, elements will be
/// recycled even if they do not repeat an even number of times.
///
/// ```rust
/// use r::r_vector::iterators::zip_recycle;
///
/// let x = vec![1, 2, 3, 4];
/// let y = vec![2, 4];
/// let z: Vec<_> = zip_recycle(x, y).collect();
/// ````
///
pub fn zip_recycle<'a, L, R, LIter, LItem, RIter, RItem>(
    l: L,
    r: R,
) -> impl Iterator<Item = (LItem, RItem)> + 'a
where
    L: IntoIterator<Item = LItem, IntoIter = LIter> + 'a,
    R: IntoIterator<Item = RItem, IntoIter = RIter> + 'a,
    LIter: ExactSizeIterator + Iterator<Item = LItem> + Clone + 'a,
    RIter: ExactSizeIterator + Iterator<Item = RItem> + Clone + 'a,
{
    let l_iter = l.into_iter();
    let r_iter = r.into_iter();
    let n = std::cmp::max(l_iter.len(), r_iter.len());
    l_iter.cycle().zip(r_iter.cycle()).take(n)
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
pub fn map_common_add_numeric<'a, I, LItem, RItem, LNum, RNum, Output>(
    i: I,
) -> impl Iterator<Item = (Output, Output)> + 'a
where
    // iterator over pairs of items
    I: IntoIterator<Item = (LItem, RItem)> + 'a,
    // and item should be coercible to numeric
    LItem: IntoNumeric<LNum> + 'a,
    LNum: 'a,
    RItem: IntoNumeric<RNum> + 'a,
    RNum: 'a,
    // and those numerics should be coercible to a common numeric
    (LNum, RNum): CommonNum<Output> + 'a,
{
    i.into_iter()
        .map(|(l, r)| (LItem::as_numeric(l), RItem::as_numeric(r)).as_common())
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
pub fn map_common_mul_numeric<'a, I, LItem, RItem, LNum, RNum, Output>(
    i: I,
) -> impl Iterator<Item = (Output, Output)> + 'a
where
    // iterator over pairs of items
    I: IntoIterator<Item = (LItem, RItem)> + 'a,
    // and item should be coercible to numeric
    LItem: IntoNumeric<LNum> + 'a,
    LNum: 'a,
    RItem: IntoNumeric<RNum> + 'a,
    RNum: 'a,
    // and those numerics should be coercible to a common numeric
    (LNum, RNum): CommonNum<Output> + 'a,
{
    i.into_iter()
        .map(|(l, r)| (LItem::as_numeric(l), RItem::as_numeric(r)).as_common())
}
