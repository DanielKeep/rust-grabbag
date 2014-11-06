#![feature(macro_rules)]

/**
This macro provides a way to initialise any container for which there is a FromIterator implementation.  It allows for both sequence and map syntax to be used, as well as inline type ascription for the result.

For example:

```
# #![feature(phase)]
# #[phase(plugin)] extern crate grabbag_macros;
# use std::collections::{HashMap, VecMap};
# fn main() {
// Initialise an empty collection.
let a: Vec<int> = collect![];
let b: HashMap<String, bool> = collect![];

// Initialise a sequence.
let c: String = collect!['a', 'b', 'c'];

// Initialise a sequence with a type constraint.
let d = collect![into Vec<_>: 0i, 1, 2];

// Initialise a map collection.
let e: VecMap<&str> = collect![1:"one", 2:"two", 3:"many", 4:"lots"];

// Initialise a map with a type constraint.
let f: HashMap<_, u8> = collect![into HashMap<int, _>: 42: 0, -11: 2];
# }
```
*/
#[macro_export]
macro_rules! collect {
    // Short-hands for initialising an empty collection.
    [] => { collect![into _] };
    [into $col_ty:ty] => { collect![into $col_ty:] };
    [into $col_ty:ty:] => {
        {
            let col: $col_ty = ::std::iter::FromIterator::from_iter(None.into_iter());
            col
        }
    };

    // Initialise a sequence with a constrained container type.
    [into $col_ty:ty: $($vs:expr),+] => {
        {
            // This is inefficient.  Ideally, we'd do all this on the stack, with the target collection being the only thing to allocate.
            let vs = vec![$($vs),+];
            let col: $col_ty = ::std::iter::FromIterator::from_iter(vs.into_iter());
            col
        }
    };

    // Initialise a sequence with a fully inferred contained type.
    [$($vs:expr),+] => { collect![into _: $($vs),+] };

    // Initialise a map with a constrained container type.
    [into $col_ty:ty: $($ks:expr: $vs:expr),+] => {
        // Maps implement FromIterator by taking tuples, so we just need to rewrite each `a:b` as `(a,b)`.
        collect![into $col_ty: $(($ks, $vs)),+]
    };

    // Initialise a map with a fully inferred contained type.
    [$($ks:expr: $vs:expr),+] => { collect![into _: $($ks: $vs),+] };
}
