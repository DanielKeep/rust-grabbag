#![feature(macro_rules)]
#![feature(phase)]
#[phase(plugin)] extern crate grabbag_macros;

use std::collections::{HashMap, TreeSet};

macro_rules! assert_eq_iter {
    (== $it:expr:) => {
        {
            assert_eq!($it.next(), None);
        }
    };
    (== $it:expr: $e:expr $(, $es:expr)*) => {
        {
            assert_eq!($it.next(), Some($e));
            assert_eq_iter!($it: $($es),*);
        }
    };
    ($it:expr: $($es:expr),*) => {
        {
            let mut it = $it;
            assert_eq_iter!(== it: $($es),*);
        }
    };
}

macro_rules! assert_eq_iter_sort {
    ($it:expr: $($es:expr),*) => {
        {
            let mut it = $it.collect::<Vec<_>>();
            it.sort();
            let mut it = it.into_iter();
            assert_eq_iter!(== it: $($es),*);
        }
    };
}

#[test]
fn test_collect_empty_full_inference() {
    let c: Vec<int> = collect![];
    assert_eq!(c.len(), 0);

    let c: String = collect![];
    assert_eq!(c.len(), 0);

    let c: HashMap<String, Vec<u8>> = collect![];
    assert_eq!(c.len(), 0);

    let c: TreeSet<int> = collect![];
    assert_eq!(c.len(), 0);
}

#[test]
fn test_collect_empty_constrained() {
    let c = collect![into Vec<int>];
    assert_eq!(c.len(), 0);

    let c = collect![into String];
    assert_eq!(c.len(), 0);

    let c = collect![into HashMap<String, Vec<u8>>];
    assert_eq!(c.len(), 0);

    let c = collect![into TreeSet<int>];
    assert_eq!(c.len(), 0);
}

#[test]
fn test_collect_sequence_full_inference() {
    let c: Vec<int> = collect![1, 2, 3];
    assert_eq_iter!(c.into_iter(): 1, 2, 3);

    let c: String = collect!['a', 'b', 'c', '刀'];
    assert_eq_iter!(c.chars(): 'a', 'b', 'c', '刀');

    let c: TreeSet<int> = collect![2, 1, 3];
    assert_eq_iter!(c.iter().map(deref): 1, 2, 3);
}

#[test]
fn test_collect_sequence_constrained() {
    let c = collect![into Vec<_>: 1i, 2, 3];
    assert_eq_iter!(c.into_iter(): 1, 2, 3);

    let c = collect![into String: 'a', 'b', 'c', '刀'];
    assert_eq_iter!(c.chars(): 'a', 'b', 'c', '刀');

    let c = collect![into TreeSet<_>: 2, 1, 3i];
    assert_eq_iter!(c.iter().map(deref): 1, 2, 3);
}

#[test]
fn test_collect_map_full_inference() {
    let c: HashMap<&str, int> = collect!["a": 0, "b": 2, "c": 42];
    assert_eq_iter_sort!(c.into_iter(): ("a", 0), ("b", 2), ("c", 42));
}

#[test]
fn test_collect_map_constrained() {
    let c = collect![into HashMap<&str, int>: "a": 0, "b": 2, "c": 42];
    assert_eq_iter_sort!(c.into_iter(): ("a", 0), ("b", 2), ("c", 42));

    let c = collect![into HashMap<_, _>: "a": 0, "b": 2, "c": 42i];
    assert_eq_iter_sort!(c.into_iter(): ("a", 0), ("b", 2), ("c", 42));
}

fn deref<T>(r: &T) -> T where T: Copy {
    *r
}
