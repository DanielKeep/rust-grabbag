/*
Copyright ⓒ 2015 grabbag contributors.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
#[macro_use] extern crate grabbag_macros;

use std::collections::{HashMap, BTreeSet};

macro_rules! assert_eq_iter {
    (== $it:expr,) => {
        {
            assert_eq!($it.next(), None);
        }
    };
    (== $it:expr, $e:expr) => {
        {
            assert_eq!($it.next(), Some($e));
        }
    };
    (== $it:expr, $e:expr, $($es:expr),*) => {
        {
            assert_eq!($it.next(), Some($e));
            assert_eq_iter!($it, $($es),*);
        }
    };
    ($it:expr, $($es:expr),*) => {
        {
            let mut it = $it;
            assert_eq_iter!(== it, $($es),*);
        }
    };
}

macro_rules! assert_eq_iter_sort {
    ($it:expr, $($es:expr),*) => {
        {
            let mut it = $it.collect::<Vec<_>>();
            it.sort();
            let mut it = it.into_iter();
            assert_eq_iter!(== it, $($es),*);
        }
    };
}

#[test]
fn test_collect_empty_full_inference() {
    let c: Vec<i32> = collect![];
    assert_eq!(c.len(), 0);

    let c: String = collect![];
    assert_eq!(c.len(), 0);

    let c: HashMap<String, Vec<u8>> = collect![];
    assert_eq!(c.len(), 0);

    let c: BTreeSet<i32> = collect![];
    assert_eq!(c.len(), 0);
}

#[test]
fn test_collect_empty_constrained() {
    let c = collect![into Vec<i32>];
    assert_eq!(c.len(), 0);

    let c = collect![into String];
    assert_eq!(c.len(), 0);

    let c = collect![into HashMap<String, Vec<u8>>];
    assert_eq!(c.len(), 0);

    let c = collect![into BTreeSet<i32>];
    assert_eq!(c.len(), 0);
}

#[test]
fn test_collect_sequence_full_inference() {
    let c: Vec<i32> = collect![1, 2, 3];
    assert_eq_iter!(c.into_iter(), 1, 2, 3);

    let c: String = collect!['a', 'b', 'c', '刀'];
    assert_eq_iter!(c.chars(), 'a', 'b', 'c', '刀');

    let c: BTreeSet<i32> = collect![2, 1, 3];
    assert_eq_iter!(c.iter().map(deref), 1, 2, 3);
}

#[test]
fn test_collect_sequence_constrained() {
    let c = collect![into Vec<_>: 1, 2, 3];
    assert_eq_iter!(c.into_iter(), 1, 2, 3);

    let c = collect![into String: 'a', 'b', 'c', '刀'];
    assert_eq_iter!(c.chars(), 'a', 'b', 'c', '刀');

    let c = collect![into BTreeSet<_>: 2, 1, 3];
    assert_eq_iter!(c.iter().map(deref), 1, 2, 3);
}

#[test]
fn test_collect_map_full_inference() {
    let c: HashMap<&str, i32> = collect!["a" => 0, "b" => 2, "c" => 42];
    assert_eq_iter_sort!(c.into_iter(), ("a", 0), ("b", 2), ("c", 42));
}

#[test]
fn test_collect_map_constrained() {
    let c = collect![into HashMap<&str, i32>: "a" => 0, "b" => 2, "c" => 42];
    assert_eq_iter_sort!(c.into_iter(), ("a", 0), ("b", 2), ("c", 42));

    let c = collect![into HashMap<_, _>: "a" => 0, "b" => 2, "c" => 42];
    assert_eq_iter_sort!(c.into_iter(), ("a", 0), ("b", 2), ("c", 42));
}

#[test]
fn test_collect_eval_once() {
    let mut n = 0;

    macro_rules! npp {
        () => ({n += 1; n});
    }

    let _: Vec<_> = collect![npp!()];
    assert_eq!(n, 1);

    n = 0;
    let _: Vec<_> = collect![npp!(), npp!()];
    assert_eq!(n, 2);

    n = 0;
    let _: Vec<_> = collect![npp!(), npp!(), npp!()];
    assert_eq!(n, 3);

    n = 0;
    let _: HashMap<_, _> = collect![npp!() => npp!()];
    assert_eq!(n, 2);

    n = 0;
    let _: HashMap<_, _> = collect![npp!() => npp!(), npp!() => npp!()];
    assert_eq!(n, 4);

    n = 0;
    let _: HashMap<_, _> = collect![npp!() => npp!(), npp!() => npp!(), npp!() => npp!()];
    assert_eq!(n, 6);
}

fn deref<T>(r: &T) -> T where T: Copy {
    *r
}
