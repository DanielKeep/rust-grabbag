# 0.1.2

* **Breaking change**: Removed `sequence!` and `recurrence!` for Rust 1.9.0+.  Due to language changes, these macros **cannot** be fixed in a backward-compatible fashion.  Existing working code should be unaffected.

* Fixed `recurrence!` macro for Rust < 1.9.0.

# 0.1.1

* Updated to 1.7.0.

# 0.1.0

* Fixed double-eval in `collect!` macro.

# 0.0.3

* Update to recent rust nightly.

# 0.0.2

* Changed `collect!` to *not* allocate temporary storage.

# 0.0.1

Initial package release.

* `collect!`
* `count_exprs!`
* `sequence!`
* `recurrence!`
