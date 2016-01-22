/*
Copyright ⓒ 2015 grabbag contributors.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
#[macro_use] extern crate grabbag_macros;

#[test]
fn test_count_exprs() {
    assert_eq!(count_exprs!(), 0);
    assert_eq!(count_exprs!(0), 1);
    assert_eq!(count_exprs!(x), 1);
    assert_eq!(count_exprs!(a*x.pow(2) + b*x + c == 0), 1);
    assert_eq!(count_exprs!(0, 1, 2), 3);
}
