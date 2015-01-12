#[macro_use] extern crate grabbag_macros;

#[test]
fn test_count_exprs() {
    assert_eq!(count_exprs!(), 0u);
    assert_eq!(count_exprs!(), 0i);
    assert_eq!(count_exprs!(0), 1u);
    assert_eq!(count_exprs!(x), 1u);
    assert_eq!(count_exprs!(a*x.pow(2) + b*x + c == 0), 1u);
    assert_eq!(count_exprs!(0, 1, 2), 3u);
}
