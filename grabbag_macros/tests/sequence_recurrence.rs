/*
Copyright ⓒ 2016 grabbag contributors.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
#![cfg(not(cannot_use_dotdotdot))]
#[macro_use] extern crate grabbag_macros;

macro_rules! iter_assert_eq {
    ($it:expr, [$($exs:expr),* $(,)*] ...) => {
        {
            let mut it = ::std::iter::IntoIterator::into_iter($it);
            let mut _i = 0;
            $(
                match (it.next(), $exs) {
                    (Some(e), ex) => {
                        if !(e == ex) {
                            panic!("assertion failed: `(left == right)` \
                                (left: `{:?}`, \
                                right: `{:?}`, \
                                element: {})", e, ex, _i);
                        }
                    },
                    (None, ex) => {
                        panic!("assertion failed: `(left == right)` \
                            (left: None, \
                            right: `{:?}`, \
                            element: {})", ex, _i);
                    }
                }
                _i += 1;
            )*
        }
    };
}

#[test]
fn test_sequence() {
    iter_assert_eq!(sequence![ n: u64 = 2*(n as u64) + 1 ], [1, 3, 5, 7]...);
    iter_assert_eq!(sequence![ a[n]: u64 = 1, 2... a[0]*(n as u64) + a[1] ], [1, 2, 4, 5, 6]...);
}

#[test]
fn test_recurrence() {
    iter_assert_eq!(recurrence![ fib[n]: f64 = 0.0, 1.0 ... fib[n-1] + fib[n-2] ],
        [0.0, 1.0, 1.0, 2.0, 3.0, 5.0, 8.0]...);
}
