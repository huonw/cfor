#![feature(macro_rules)]

//! A C-style `for` loop in macro form.
//!
//! This takes the form `cfor!(initialiser; condition; step {
//! body })`.
//!
//! - `initialiser` is a statement evaluated before any iterations of
//!   the loop. Any variables declared here are scoped to the `cfor!`
//!   invocation, that is, only usable inside `condition`, `step` and
//!   `body`.
//! - `condition` is an boolean expression evaluated at the start of
//!   each iteration. If it evaluates to `false` iteration will stop.
//! - `step` is an arbitrary expression which is executed at the end
//!   of each iteration (including if `continue` is called), before
//!   `condition` is checked.
//!
//!
//! The initialiser and condition can be empty like C, but the step
//! cannot unlike C. A `for` loop with no step is identical to a
//! `while` loop.
//!
//! # When should I use it?
//!
//! *Only* when `cfor!` is clearer than the more declarative built-in
//! [iterators](http://doc.rust-lang.org/master/std/iter/), [their
//! adaptors](http://doc.rust-lang.org/master/std/iter/trait.Iterator.html)
//! and the `for` loop. For example, the built-in iterator
//! functionality is more self-contained so there is less risk of
//! accidentally writing `i` in a condition when `j` was meant (I
//! personally get bitten by this semiregularly when writing nested
//! "2D" loops in C).
//!
//! Furthermore, the adaptor methods linked above allow [one to
//! write](http://huonw.github.io/2014/06/10/knn-rust.html) concise,
//! performant, reusable "functional" code in a way that is not
//! possible to achieve using C-style iteration.
//!
//! # How to use it?
//!
//! Add the repository as a normal cargo dependency, and include into
//! your crate with `#[phase(plugin)]`. (See examples below.)
//!
//! ```toml
//! [dependencies.cfor]
//! git = "https://github.com/huonw/cfor"
//! ```
//!
//! # Examples
//!
//! ## Simple
//!
//! A non-additive condition is not handled extremely naturally by
//! `std::iter`, but is straight-forward to handle directly.
//!
//! ```rust
//! #![feature(phase)]
//! #[phase(plugin)] extern crate cfor;
//!
//! fn main() {
//!     cfor!{let mut x = 1u; x < 0x1000; x *= 2 {
//!         println!("power of 2: {}", x);
//!     }}
//! }
//! ```
//!
//! ## Intrabody condition
//!
//! If a condition requires some extra computation to be checked (or
//! if there is some code that should always be evaluated, even if the
//! condition will be `false` for a given iteration), the condition in
//! the `cfor` header can be omitted.
//!
//! ```rust
//! #![feature(phase)]
//! #[phase(plugin)] extern crate cfor;
//!
//! fn main() {
//!     cfor!{let mut x = 1u; ; x *= 2 {
//!         // ... setup ...
//!         println!("handling power of 2: {}", x);
//!
//!         if x < 0x1000 { break }
//!
//!         // ... further handling ...
//!         println!("handling power of 2: {}", x);
//!     }}
//! }
//! ```
//!
//! ## Out-of-loop initialisation
//!
//! Sometimes one may wish to have access to a variable outside the
//! loop after it finishes so it has to be declared outside the loop,
//! or one may be iterating over some presupplied/-computed value so
//! there is no meaningful additional initialisation possible. The
//! initialisation expression can be safely omitted in this case.
//!
//! ```rust
//! #![feature(phase)]
//! #[phase(plugin)] extern crate cfor;
//!
//! use std::rand;
//!
//! fn main() {
//!     let mut x = 1u16;
//!
//!     cfor!{; x < 0x1000; x *= 2 {
//!         println!("power of 2: {}", x);
//!
//!         // sometimes quit early
//!         if x > rand::random() { break }
//!     }}
//!
//!     println!("actually stopped at {}", x);
//! }
//! ```
//!
//! # Handling `continue`
//!
//! (Or, "why is the macro so complicated?")
//!
//! Special effort is made to ensure that `continue` acts correctly, a
//! naive macro defined as follows will cause `continue` to also skip
//! evaluating `step`, likely leading to undesirable behaviour like
//! infinite loops.
//!
//! ```rust
//! #![feature(macro_rules)]
//!
//! // WARNING: this is broken.
//! macro_rules! bad_cfor {
//!     ($init: stmt; $cond: expr; $step: expr $body: block) => {
//!         {
//!             $init;
//!             while $cond {
//!                 $body;
//!
//!                 $step;
//!             }
//!         }
//!     }
//! }
//!
//! fn main() {
//!     let mut true_counter = 0u;
//!
//!     bad_cfor!{let mut i = 0u; i < 10; i += 1 {
//!
//!         // manually avoid the infinite loop
//!         if true_counter >= 50 { break }
//!         true_counter += 1;
//!
//!         println!("i = {}", i);
//!         // try to skip just i == 4
//!         if i == 4 {
//!             continue
//!         }
//!         // ...more code...
//!     }}
//! }
//! ```
//!
//! This is invoked in the same manner as `cfor!`, but, if `$body`
//! contains a `continue`, the `$step` at the end of the loop body
//! will never be evaluated.


/// A C-style `for` loop in macro form.
///
/// See crates docs for more information
#[macro_export]
pub macro_rules! cfor {
    // for (; ...; ...) { ... }
    (; $($rest: tt)*) => {
        cfor!((); $($rest)*)
    };
    // for ($init; ; ...) { ... }
    ($init: stmt; ; $($rest: tt)*) => {
        cfor!($init; true; $($rest)*)
    };

    // for ($init; $cond; $step) { $body }
    ($init: stmt; $cond: expr; $step: expr $body: block) => {
        {
            let mut _first = true;
            $init;
            while {
                if _first {
                    _first = false
                } else {
                    $step
                }

                $cond
            } $body
        }
    };
}
