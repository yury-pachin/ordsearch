//! This crate provides an efficient data structure for approximate lookups in ordered collections.
//!
//! More concretely, given a set `A` of `n` values, and a query value `x`, this library provides an
//! efficient mechanism for finding the smallest value in `A` that is greater than or equal to `x`.
//! In particular, this library caters to the important case where there are many such queries to
//! the same array, `A`.
//!
//! This library is constructed from the best solution identified in [Array Layouts for
//! Comparison-Based Searching](https://arxiv.org/abs/1509.05053) by Paul-Virak Khuong and Pat
//! Morin. For more information, see the paper, [their
//! website](http://cglab.ca/~morin/misc/arraylayout-v2/), and the [C++ implementation
//! repository](https://github.com/patmorin/arraylayout).
//!
//! # Current implementation
//!
//! At the time of writing, this implementation uses a branch-free search over an
//! Eytzinger-arranged array with masked prefetching based on the [C++
//! implementation](https://github.com/patmorin/arraylayout/blob/3f20174a2a0ab52c6f37f2ea87d087307f19b5ee/src/eytzinger_array.h#L253)
//! written by the authors of the aforementioned paper. This is the recommended algorithm from the
//! paper, and what the authors suggested in
//! https://github.com/patmorin/arraylayout/issues/3#issuecomment-338472755.
//!
//! Note that prefetching is *only* enabled with the (non-default) `nightly` feature due to
//! https://github.com/aweinstock314/prefetch/issues/1. Suggestions for workarounds welcome.
//!
//! # Performance
//!
//! The included benchmarks can be run with
//!
//! ```console,ignore
//! $ cargo +nightly bench --features nightly
//! ```
//!
//! This will benchmark both construction and search with different number of values, and
//! differently sized values -- look for the line that aligns closest with your data. The general
//! trend is that `ordsearch` is faster when `n` is smaller and `T` is larger. You may also want to
//! compare with the pending Rust PR "[Improve SliceExt::binary_search
//! performance](https://github.com/rust-lang/rust/pull/45333)".
//! [Summarized](https://github.com/BurntSushi/cargo-benchcmp) results from my laptop (an X1 Carbon
//! with i7-5600U @ 2.60GHz) are given below.
//!
//! Compared to binary search over a sorted vector:
//!
//! ```diff,ignore
//!  name           sorted_vec ns/iter  this ns/iter  diff ns/iter   diff %  speedup
//! +u8::l1         46                  37                      -9  -19.57%   x 1.24
//! -u8::l1_dup     31                  37                       6   19.35%   x 0.84
//! -u8::l2         44                  58                      14   31.82%   x 0.76
//! -u8::l2_dup     31                  56                      25   80.65%   x 0.55
//! -u8::l3         29                  170                    141  486.21%   x 0.17
//! -u8::l3_dup     30                  127                     97  323.33%   x 0.24
//! +u32::l1        66                  37                     -29  -43.94%   x 1.78
//! +u32::l1_dup    41                  37                      -4   -9.76%   x 1.11
//! +u32::l2        85                  64                     -21  -24.71%   x 1.33
//! -u32::l2_dup    62                  64                       2    3.23%   x 0.97
//! -u32::l3        180                 380                    200  111.11%   x 0.47
//! -u32::l3_dup    156                 381                    225  144.23%   x 0.41
//! +usize::l1      66                  37                     -29  -43.94%   x 1.78
//! +usize::l1_dup  41                  37                      -4   -9.76%   x 1.11
//! +usize::l2      87                  67                     -20  -22.99%   x 1.30
//! -usize::l2_dup  62                  77                      15   24.19%   x 0.81
//! -usize::l3      247                 522                    275  111.34%   x 0.47
//! -usize::l3_dup  203                 614                    411  202.46%   x 0.33
//! ```
//!
//! Compared to a `BTreeSet`:
//!
//! ```diff,ignore
//!  name           btreeset ns/iter  this ns/iter  diff ns/iter   diff %  speedup
//! +u8::l1         48                37                     -11  -22.92%   x 1.30
//! -u8::l1_dup     35                37                       2    5.71%   x 0.95
//! -u8::l2         46                58                      12   26.09%   x 0.79
//! -u8::l2_dup     35                56                      21   60.00%   x 0.62
//! -u8::l3         36                170                    134  372.22%   x 0.21
//! -u8::l3_dup     34                127                     93  273.53%   x 0.27
//! +u32::l1        66                37                     -29  -43.94%   x 1.78
//! +u32::l1_dup    42                37                      -5  -11.90%   x 1.14
//! +u32::l2        91                64                     -27  -29.67%   x 1.42
//! -u32::l2_dup    60                64                       4    6.67%   x 0.94
//! -u32::l3        351               380                     29    8.26%   x 0.92
//! -u32::l3_dup    195               381                    186   95.38%   x 0.51
//! +usize::l1      66                37                     -29  -43.94%   x 1.78
//! +usize::l1_dup  42                37                      -5  -11.90%   x 1.14
//! +usize::l2      96                67                     -29  -30.21%   x 1.43
//! -usize::l2_dup  61                77                      16   26.23%   x 0.79
//! -usize::l3      441               522                     81   18.37%   x 0.84
//! -usize::l3_dup  241               614                    373  154.77%   x 0.39
//! ```
//!
//! # Future work
//!
//!  - [ ] Implement aligned operation: https://github.com/patmorin/arraylayout/blob/3f20174a2a0ab52c6f37f2ea87d087307f19b5ee/src/eytzinger_array.h#L204
//!  - [ ] Implement deep prefetching for large `T`: https://github.com/patmorin/arraylayout/blob/3f20174a2a0ab52c6f37f2ea87d087307f19b5ee/src/eytzinger_array.h#L128
//!
#![deny(missing_docs)]
#![cfg_attr(feature = "nightly", feature(test))]
#![cfg_attr(feature = "nightly", feature(concat_idents))]
#[cfg(feature = "nightly")]
extern crate prefetch;
#[cfg(feature = "nightly")]
extern crate test;

use std::borrow::Borrow;

/// A collection of ordered items that can efficiently satisfy queries for nearby elements.
///
/// The most interesting method here is `find_gte`.
///
/// # Examples
///
/// ```
/// # use ordsearch::OrderedCollection;
/// let x = OrderedCollection::from(vec![1, 2, 4, 8, 16, 32, 64]);
/// assert_eq!(x.find_gte(0), Some(&1));
/// assert_eq!(x.find_gte(1), Some(&1));
/// assert_eq!(x.find_gte(3), Some(&4));
/// assert_eq!(x.find_gte(6), Some(&8));
/// assert_eq!(x.find_gte(8), Some(&8));
/// assert_eq!(x.find_gte(64), Some(&64));
/// assert_eq!(x.find_gte(65), None);
/// ```
pub struct OrderedCollection<T> {
    items: Vec<T>,

    #[cfg(feature = "nightly")] mask: usize,
}

impl<T: Ord> From<Vec<T>> for OrderedCollection<T> {
    /// Construct a new `OrderedCollection` from a vector of elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ordsearch::OrderedCollection;
    /// let a = OrderedCollection::from(vec![42, 89, 7, 12]);
    /// assert_eq!(a.find_gte(50), Some(&89));
    /// ```
    fn from(mut v: Vec<T>) -> OrderedCollection<T> {
        v.sort_unstable();
        Self::from_sorted_iter(v.into_iter())
    }
}

/// Insert items from the sorted iterator `iter` into `v` in complete binary tree order.
///
/// Requires `iter` to be a sorted iterator.
/// Requires v's capacity to be set to the number of elements in `iter`.
/// The length of `v` will not be changed by this function.
fn eytzinger_walk<I, T>(v: &mut Vec<T>, iter: &mut I, i: usize)
where
    I: Iterator<Item = T>,
{
    if i >= v.capacity() {
        return;
    }

    // visit left child
    eytzinger_walk(v, iter, 2 * i + 1);

    // put data at the root
    // we know the get_unchecked_mut and unwrap below are safe because we set the Vec's capacity to
    // the length of the iterator.
    *unsafe { v.get_unchecked_mut(i) } = iter.next().unwrap();

    // visit right child
    eytzinger_walk(v, iter, 2 * i + 2);
}

impl<T: Ord> OrderedCollection<T> {
    /// Construct a new `OrderedCollection` from an iterator over sorted elements.
    ///
    /// Note that if the iterator is *not* sorted, no error will be given, but lookups will give
    /// incorrect results. The given iterator must also implement `ExactSizeIterator` so that we
    /// know the size of the lookup array.
    ///
    /// # Examples
    ///
    /// Using an already-sorted iterator:
    ///
    /// ```
    /// # use std::collections::BTreeSet;
    /// # use ordsearch::OrderedCollection;
    ///
    /// let mut s = BTreeSet::new();
    /// s.insert(42);
    /// s.insert(89);
    /// s.insert(7);
    /// s.insert(12);
    /// let a = OrderedCollection::from_sorted_iter(s);
    /// assert_eq!(a.find_gte(50), Some(&89));
    /// ```
    ///
    /// Sorting a collection and then iterating (in this case, you'd likely use `new` instead):
    ///
    /// ```
    /// # use ordsearch::OrderedCollection;
    /// let mut v = vec![42, 89, 7, 12];
    /// v.sort_unstable();
    /// let a = OrderedCollection::from_sorted_iter(v);
    /// assert_eq!(a.find_gte(50), Some(&89));
    /// ```
    ///
    /// The `OrderedCollection` can also be over references to somewhere else:
    ///
    /// ```
    /// # use std::collections::BTreeSet;
    /// # use ordsearch::OrderedCollection;
    ///
    /// let mut s = BTreeSet::new();
    /// s.insert(42);
    /// s.insert(89);
    /// s.insert(7);
    /// s.insert(12);
    /// let a = OrderedCollection::from_sorted_iter(s.iter());
    /// assert_eq!(a.find_gte(50), Some(&&89));
    /// ```
    ///
    pub fn from_sorted_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator<Item = T>,
    {
        let mut iter = iter.into_iter();
        let n = iter.len();
        let mut v = Vec::with_capacity(n);
        eytzinger_walk(&mut v, &mut iter, 0);

        // it's now safe to set the length, since all `n` elements have been inserted.
        unsafe { v.set_len(n) };

        #[cfg(feature = "nightly")]
        {
            let mut mask = 1;
            while mask <= n {
                mask <<= 1;
            }
            mask -= 1;

            OrderedCollection {
                items: v,
                mask: mask,
            }
        }
        #[cfg(not(feature = "nightly"))]
        OrderedCollection { items: v }
    }

    /// Construct a new `OrderedCollection` from a slice of elements.
    ///
    /// Note that the underlying slice will be reordered!
    ///
    /// # Examples
    ///
    /// ```
    /// # use ordsearch::OrderedCollection;
    /// let mut vals = [42, 89, 7, 12];
    /// let a = OrderedCollection::from_slice(&mut vals);
    /// assert_eq!(a.find_gte(50), Some(&&89));
    /// ```
    pub fn from_slice<'a>(v: &'a mut [T]) -> OrderedCollection<&'a T> {
        v.sort_unstable();
        OrderedCollection::from_sorted_iter(v.into_iter().map(|x| &*x))
    }

    /// Find the smallest value `v` such that `v >= x`.
    ///
    /// Returns `None` if there is no such `v`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ordsearch::OrderedCollection;
    /// let x = OrderedCollection::from(vec![1, 2, 4, 8, 16, 32, 64]);
    /// assert_eq!(x.find_gte(0), Some(&1));
    /// assert_eq!(x.find_gte(1), Some(&1));
    /// assert_eq!(x.find_gte(3), Some(&4));
    /// assert_eq!(x.find_gte(6), Some(&8));
    /// assert_eq!(x.find_gte(8), Some(&8));
    /// assert_eq!(x.find_gte(64), Some(&64));
    /// assert_eq!(x.find_gte(65), None);
    /// ```
    pub fn find_gte<'a, X>(&'a self, x: X) -> Option<&'a T>
    where
        T: Borrow<X>,
        X: Ord,
    {
        use std::mem;

        let mut i = 0;
        let multiplier = 64 / mem::size_of::<T>();
        let offset = multiplier + multiplier / 2;
        let _ = offset; // avoid warning about unused w/o nightly

        while i < self.items.len() {
            #[cfg(feature = "nightly")]
            {
                use prefetch::prefetch::*;
                // unsafe is safe because pointer is never dereferenced
                unsafe {
                    prefetch::<Read, High, Data, _>(
                        self.items
                            .as_ptr()
                            .offset(((multiplier * i + offset) & self.mask) as isize),
                    )
                };
            }

            // safe because i < self.items.len()
            i = if x.borrow() <= unsafe { self.items.get_unchecked(i) }.borrow() {
                2 * i + 1
            } else {
                2 * i + 2
            };
        }

        // we want ffs(~(i + 1))
        // since ctz(x) = ffs(x) - 1
        // we use ctz(~(i + 1)) + 1
        let j = (i + 1) >> ((!(i + 1)).trailing_zeros() + 1);
        if j == 0 {
            None
        } else {
            Some(unsafe { self.items.get_unchecked(j - 1) })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OrderedCollection;

    #[test]
    fn complete_exact() {
        let x = OrderedCollection::from(vec![1, 2, 4, 8, 16, 32, 64]);
        assert_eq!(x.find_gte(1), Some(&1));
        assert_eq!(x.find_gte(2), Some(&2));
        assert_eq!(x.find_gte(4), Some(&4));
        assert_eq!(x.find_gte(8), Some(&8));
        assert_eq!(x.find_gte(16), Some(&16));
        assert_eq!(x.find_gte(32), Some(&32));
        assert_eq!(x.find_gte(64), Some(&64));
    }

    #[test]
    fn complete_approximate() {
        let x = OrderedCollection::from(vec![1, 2, 4, 8, 16, 32, 64]);
        assert_eq!(x.find_gte(0), Some(&1));
        assert_eq!(x.find_gte(3), Some(&4));
        assert_eq!(x.find_gte(5), Some(&8));
        assert_eq!(x.find_gte(6), Some(&8));
        assert_eq!(x.find_gte(7), Some(&8));
        for i in 9..16 {
            assert_eq!(x.find_gte(i), Some(&16));
        }
        for i in 17..32 {
            assert_eq!(x.find_gte(i), Some(&32));
        }
        for i in 33..64 {
            assert_eq!(x.find_gte(i), Some(&64));
        }
        assert_eq!(x.find_gte(65), None);
    }

    #[test]
    fn unbalanced_exact() {
        let x = OrderedCollection::from(vec![1, 2, 4, 8, 16, 32, 64, 128, 256]);
        assert_eq!(x.find_gte(1), Some(&1));
        assert_eq!(x.find_gte(2), Some(&2));
        assert_eq!(x.find_gte(4), Some(&4));
        assert_eq!(x.find_gte(8), Some(&8));
        assert_eq!(x.find_gte(16), Some(&16));
        assert_eq!(x.find_gte(32), Some(&32));
        assert_eq!(x.find_gte(64), Some(&64));
        assert_eq!(x.find_gte(128), Some(&128));
        assert_eq!(x.find_gte(256), Some(&256));
    }

    #[test]
    fn unbalanced_approximate() {
        let x = OrderedCollection::from(vec![1, 2, 4, 8, 16, 32, 64, 128, 256]);
        assert_eq!(x.find_gte(0), Some(&1));
        assert_eq!(x.find_gte(3), Some(&4));
        assert_eq!(x.find_gte(5), Some(&8));
        assert_eq!(x.find_gte(6), Some(&8));
        assert_eq!(x.find_gte(7), Some(&8));
        for i in 9..16 {
            assert_eq!(x.find_gte(i), Some(&16));
        }
        for i in 17..32 {
            assert_eq!(x.find_gte(i), Some(&32));
        }
        for i in 33..64 {
            assert_eq!(x.find_gte(i), Some(&64));
        }
        for i in 65..128 {
            assert_eq!(x.find_gte(i), Some(&128));
        }
        for i in 129..256 {
            assert_eq!(x.find_gte(i), Some(&256));
        }
        assert_eq!(x.find_gte(257), None);
    }
}

#[cfg(all(feature = "nightly", test))]
mod b {
    use super::OrderedCollection;
    use test::Bencher;
    use test::black_box;
    use std::collections::BTreeSet;

    // these benchmarks borrow from https://github.com/rust-lang/rust/pull/45333

    enum Cache {
        L1,
        L2,
        L3,
    }

    impl Cache {
        pub fn size(&self) -> usize {
            match *self {
                Cache::L1 => 1000,      // 8kb
                Cache::L2 => 10_000,    // 80kb
                Cache::L3 => 1_000_000, // 8Mb
            }
        }
    }

    #[inline]
    fn nodup_usize(i: usize) -> usize {
        i * 2
    }

    #[inline]
    fn nodup_u8(i: usize) -> u8 {
        nodup_usize(i) as u8
    }

    #[inline]
    fn nodup_u32(i: usize) -> u32 {
        nodup_usize(i) as u32
    }

    #[inline]
    fn dup_usize(i: usize) -> usize {
        i / 16 * 16
    }

    #[inline]
    fn dup_u8(i: usize) -> u8 {
        dup_usize(i) as u8
    }

    #[inline]
    fn dup_u32(i: usize) -> u32 {
        dup_usize(i) as u32
    }

    macro_rules! construction_benches {
        ($t:ident, $v:ident) => {
            mod $v {
                use super::*;
                fn nodup(c: Cache, b: &mut Bencher) {
                    let mk = concat_idents!(make_, $t);
                    let mapper = concat_idents!(nodup_, $v);
                    bench_construction!(c, mk, mapper, b);
                }

                #[bench]
                fn l1(b: &mut Bencher) {
                    nodup(Cache::L1, b);
                }

                #[bench]
                fn l2(b: &mut Bencher) {
                    nodup(Cache::L2, b);
                }

                fn dup(c: Cache, b: &mut Bencher) {
                    let mk = concat_idents!(make_, $t);
                    let mapper = concat_idents!(dup_, $v);
                    bench_construction!(c, mk, mapper, b);
                }

                #[bench]
                fn l1_dup(b: &mut Bencher) {
                    dup(Cache::L1, b);
                }

                #[bench]
                fn l2_dup(b: &mut Bencher) {
                    dup(Cache::L2, b);
                }
            }
        }
    }

    macro_rules! search_benches {
        ($t:ident, $v:ident) => {
            mod $v {
                use super::*;
                fn nodup(c: Cache, b: &mut Bencher) {
                    let mk = concat_idents!(make_, $t);
                    let s = concat_idents!(search_, $t);
                    let mapper = concat_idents!(nodup_, $v);
                    bench_search!(c, mk, s, mapper, b);
                }

                #[bench]
                fn l1(b: &mut Bencher) {
                    nodup(Cache::L1, b);
                }

                #[bench]
                fn l2(b: &mut Bencher) {
                    nodup(Cache::L2, b);
                }

                #[bench]
                fn l3(b: &mut Bencher) {
                    nodup(Cache::L3, b);
                }

                fn dup(c: Cache, b: &mut Bencher) {
                    let mk = concat_idents!(make_, $t);
                    let s = concat_idents!(search_, $t);
                    let mapper = concat_idents!(dup_, $v);
                    bench_search!(c, mk, s, mapper, b);
                }

                #[bench]
                fn l1_dup(b: &mut Bencher) {
                    dup(Cache::L1, b);
                }

                #[bench]
                fn l2_dup(b: &mut Bencher) {
                    dup(Cache::L2, b);
                }

                #[bench]
                fn l3_dup(b: &mut Bencher) {
                    dup(Cache::L3, b);
                }
            }
        }
    }

    macro_rules! benches {
        ($t:ident) => {
            mod $t {
                pub use super::*;
                mod construction {
                    pub use super::*;
                    construction_benches!($t, u8);
                    construction_benches!($t, u32);
                    construction_benches!($t, usize);
                }
                mod search {
                    pub use super::*;
                    search_benches!($t, u8);
                    search_benches!($t, u32);
                    search_benches!($t, usize);
                }
            }
        }
    }

    macro_rules! bench_construction {
        ($cache:expr, $make:ident, $mapper:ident, $b:ident) => {
            let size = $cache.size();
            let mut v: Vec<_> = (0..size).map(&$mapper).collect();
            let mut r = 0usize;

            $b.iter(|| {
                for e in v.iter_mut() {
                    r = r.wrapping_mul(1664525).wrapping_add(1013904223);
                    *e = $mapper(r % size);
                }
                black_box($make(&mut v));
            });
        }
    }

    macro_rules! bench_search {
        ($cache:expr, $make:ident, $search:ident, $mapper:ident, $b:ident) => {
            let size = $cache.size();
            let mut v: Vec<_> = (0..size).map(&$mapper).collect();
            let mut r = 0usize;

            let c = $make(&mut v);
            $b.iter(move || {
                // LCG constants from https://en.wikipedia.org/wiki/Numerical_Recipes.
                r = r.wrapping_mul(1664525).wrapping_add(1013904223);
                // Lookup the whole range to get 50% hits and 50% misses.
                let x = $mapper(r % size);

                black_box($search(&c, x).is_some());
            });
        }
    }

    fn make_this<T: Ord>(v: &mut Vec<T>) -> OrderedCollection<&T> {
        OrderedCollection::from_slice(v)
    }

    fn search_this<'a, T: Ord>(c: &OrderedCollection<&'a T>, x: T) -> Option<&'a T> {
        c.find_gte(x).map(|v| &**v)
    }

    benches!(this);

    fn make_btreeset<T: Ord>(v: &mut Vec<T>) -> BTreeSet<&T> {
        use std::iter::FromIterator;
        BTreeSet::from_iter(v.iter())
    }

    fn search_btreeset<'a, T: Ord>(c: &BTreeSet<&'a T>, x: T) -> Option<&'a T> {
        use std::collections::Bound;
        c.range((Bound::Included(x), Bound::Unbounded))
            .next()
            .map(|v| &**v)
    }

    benches!(btreeset);

    fn make_sorted_vec<T: Ord>(v: &mut Vec<T>) -> &[T] {
        v.sort_unstable();
        &v[..]
    }

    fn search_sorted_vec<'a, T: Ord>(c: &'a &[T], x: T) -> Option<&'a T> {
        c.binary_search(&x).ok().map(|i| &c[i])
    }

    benches!(sorted_vec);
}
