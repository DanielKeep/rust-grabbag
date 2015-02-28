/*!
This is a random grab-bag of macros that didn't really seem to deserve a separate home.

If you see something you think you have a better place for, let me know!
*/

/**
Counts the number of comma-delimited expressions passed to it.  The result is a compile-time evaluable expression, suitable for use as a static array size, or the value of a `const`.

Example:

```
# #[macro_use] extern crate grabbag_macros;
# fn main() {
const COUNT: usize = count_exprs!(a, 5+1, "hi there!".into_string());
assert_eq!(COUNT, 3);
# }
```
*/
#[macro_export]
macro_rules! count_exprs {
    () => { 0 };
    ($e:expr) => { 1 };
    ($e:expr, $($es:expr),+) => { 1 + count_exprs!($($es),*) };
}

/**
This macro provides a way to initialise any container for which there is a FromIterator implementation.  It allows for both sequence and map syntax to be used, as well as inline type ascription for the result.

For example:

```
# #[macro_use] extern crate grabbag_macros;
# use std::collections::{HashMap, VecMap};
# fn main() {
// Initialise an empty collection.
let a: Vec<int> = collect![];
let b: HashMap<String, bool> = collect![];

// Initialise a sequence.
let c: String = collect!['a', 'b', 'c'];

// Initialise a sequence with a type constraint.
let d = collect![into Vec<_>: 0, 1, 2];

// Initialise a map collection.
let e: VecMap<&str> = collect![1 => "one", 2 => "two", 3 => "many", 4 => "lots"];

// Initialise a map with a type constraint.
let f: HashMap<_, u8> = collect![into HashMap<int, _>: 42 => 0, -11 => 2];
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
            let col: $col_ty = ::std::default::Default::default();
            col
        }
    };

    // Initialise a sequence with a constrained container type.
    [into $col_ty:ty: $($vs:expr),+] => {
        {
            use std::marker::PhantomData;

            const NUM_ELEMS: usize = count_exprs!($($vs),+);

            // This trick is stolen from std::iter, and *should* serve to give the container enough information to pre-allocate sufficient storage for all the elements.
            struct SizeHint<E>(PhantomData<E>);

            impl<E> SizeHint<E> {
                // This method is needed to help the compiler work out which `Extend` impl to use in cases where there is more than one (e.g. `String`).
                #[inline(always)]
                fn type_hint(_: &[E]) -> SizeHint<E> { SizeHint(PhantomData) }
            }

            impl<E> Iterator for SizeHint<E> {
                type Item = E;

                #[inline(always)]
                fn next(&mut self) -> Option<E> {
                    None
                }

                #[inline(always)]
                fn size_hint(&self) -> (usize, Option<usize>) {
                    (NUM_ELEMS, Some(NUM_ELEMS))
                }
            }

            let mut col: $col_ty = ::std::default::Default::default();

            Extend::extend(&mut col, SizeHint::type_hint(&[$($vs),+]));

            $(Extend::extend(&mut col, Some($vs).into_iter());)+

            col
        }
    };

    // Initialise a sequence with a fully inferred contained type.
    [$($vs:expr),+] => { collect![into _: $($vs),+] };

    // Initialise a map with a constrained container type.
    [into $col_ty:ty: $($ks:expr => $vs:expr),+] => {
        // Maps implement FromIterator by taking tuples, so we just need to rewrite each `a:b` as `(a,b)`.
        collect![into $col_ty: $(($ks, $vs)),+]
    };

    // Initialise a map with a fully inferred contained type.
    [$($ks:expr => $vs:expr),+] => { collect![into _: $($ks => $vs),+] };
}

/**
Expands to an expression implementing the `Iterator` trait, which yields successive
elements of the given closed-form sequence.

For example, you can define the sequence of positive odd integers like so:

```
# #[macro_use] extern crate grabbag_macros;
# fn main() {
#     let _ =
sequence![ n: u64 = 2*(n as u64) + 1 ]
#     ;
# }
```

You can also specify one or more initial members of the sequence that are also used in the closed form expression like so:

```
# #[macro_use] extern crate grabbag_macros;
# fn main() {
#     let _ =
sequence![ a[n]: u64 = 1, 2... a[0]*(n as u64) + a[1] ]
#     ;
# }
```
*/
#[macro_export]
macro_rules! sequence {
    ( $ind:ident: $sty:ty = $closed_form:expr ) => {
        {
            struct Sequence {
                pos: usize,
            }

            impl Iterator for Sequence {
                type Item = $sty;

                #[inline]
                fn next(&mut self) -> Option<$sty> {
                    if self.pos == ::std::usize::MAX {
                        return None
                    }

                    let next_val: $sty = {
                        let $ind = self.pos;
                        $closed_form
                    };

                    self.pos += 1;
                    Some(next_val)
                }
            }

            Sequence { pos: 0 }
        }
    };
    ( $seq:ident [ $ind:ident ]: $sty:ty = $($inits:expr),+ ... $closed_form:expr ) => {
        {
            const INITS: usize = count_exprs!($($inits),+);

            struct Sequence {
                inits: [$sty; INITS],
                pos: usize,
            }

            impl Iterator for Sequence {
                type Item = $sty;

                #[inline]
                fn next(&mut self) -> Option<$sty> {
                    if self.pos == ::std::usize::MAX {
                        return None
                    }

                    if self.pos < INITS {
                        let next_val = self.inits[self.pos];
                        self.pos += 1;
                        Some(next_val)
                    } else {
                        let next_val: $sty = {
                            let $ind = self.pos;
                            let $seq = &self.inits;
                            $closed_form
                        };

                        self.pos += 1;
                        Some(next_val)
                    }
                }
            }

            Sequence { inits: [$($inits),+], pos: 0 }
        }
    };
}

/**
Expands to an expression implementing the `Iterator` trait, which yields successive
elements of the given recurrence relationship.

For example, you can define a Fibonnaci sequence iterator like so:

```
# #[macro_use] extern crate grabbag_macros;
# fn main() {
#     let _ =
recurrence![ fib[n]: f64 = 0.0, 1.0 ... fib[n-1] + fib[n-2] ]
#     ;
# }
```
*/
#[macro_export]
macro_rules! recurrence {
    ( $seq:ident [ $ind:ident ]: $sty:ty = $($inits:expr),+ ... $recur:expr ) => {
        {
            use std::ops::Index;

            const MEMORY: usize = count_exprs!($($inits),+);

            struct Recurrence {
                mem: [$sty; MEMORY],
                pos: usize,
            }

            struct IndexOffset<'a> {
                slice: &'a [$sty; MEMORY],
                offset: usize,
            }

            impl<'a> Index<usize> for IndexOffset<'a> {
                type Output = $sty;

                #[inline(always)]
                fn index<'b>(&'b self, index: &usize) -> &'b $sty {
                    let real_index = *index - self.offset + MEMORY;
                    &self.slice[real_index]
                }
            }

            impl Iterator for Recurrence {
                type Item = $sty;

                #[inline]
                fn next(&mut self) -> Option<$sty> {
                    if self.pos == ::std::usize::MAX {
                        return None
                    }

                    if self.pos < MEMORY {
                        let next_val = self.mem[self.pos];
                        self.pos += 1;
                        Some(next_val)
                    } else {
                        let next_val: $sty = {
                            let $ind = self.pos;
                            let $seq = IndexOffset { slice: &self.mem, offset: $ind };
                            $recur
                        };

                        {
                            use std::mem::swap;

                            let mut swap_tmp = next_val;
                            for i in range(0, MEMORY).rev() {
                                swap(&mut swap_tmp, &mut self.mem[i]);
                            }
                        }

                        self.pos += 1;
                        Some(next_val)
                    }
                }
            }

            Recurrence { mem: [$($inits),+], pos: 0 }
        }
    };
}
