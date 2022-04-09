use std::ops::AddAssign;

/// Expecting some SIMD optimizations.
#[inline]
pub(crate) fn vec_add_assign<L, R>(lhs: &mut [L], rhs: &[R])
where
    L: Copy + AddAssign<R>,
    R: Copy,
{
    lhs.iter_mut().zip(rhs).for_each(|(l, &r)| *l += r);
}
