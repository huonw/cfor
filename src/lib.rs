//! A C-style `for` loop in macro form.
//!
//! This takes the form `cfor!(initialiser; condition; step { body })`.
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
//! [*Source & issue tracker*](https://github.com/huonw/cfor/)
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
//! #[macro_use] extern crate cfor;
//!
//! fn main() {
//!     cfor!{let mut x = 1; x < 0x1000; x *= 2; {
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
//! #[macro_use] extern crate cfor;
//!
//! fn main() {
//!     cfor!{let mut x = 1; ; x *= 2; {
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
//! #[macro_use] extern crate cfor;
//!
//! extern crate rand;
//!
//! fn main() {
//!     let mut x = 1u16;
//!
//!     cfor!{; x < 0x1000; x *= 2; {
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
//! # // avoid our crate being inserted automatically, which gets in
//! # // the way of the feature above.
//! # #[macro_use] extern crate cfor;
//! // WARNING: this is broken.
//! macro_rules! bad_cfor {
//!     ($init: stmt; $cond: expr; $step: expr; $body: block) => {
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
//!     let mut true_counter = 0;
//!
//!     bad_cfor!{let mut i = 0; i < 10; i += 1; {
//!
//!         // manually avoid the infinite loop
//!         if true_counter >= 50 { break }
//!         true_counter += 1;
//!
//!         println!("i = {}", i);
//!         // try to skip just i == 4
//!         if i == 4 {
//!             // but this skips the i += 1 leaving us
//!             // on i == 4 forever.
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
/// See crates docs for more information.
#[macro_export]
macro_rules! cfor {
    // for (; ...; ...) { ... }
    (; $($rest: tt)*) => {
        cfor!((); $($rest)*)
    };
    // for ($init; ; ...) { ... }
    ($init: stmt; ; $($rest: tt)*) => {
        // avoid the `while true` lint
        cfor!($init; !false; $($rest)*)
    };

    // for ($init; $cond; ) { ... }
    ($init: stmt; $cond: expr; ; $body: block) => {
        cfor!{$init; $cond; (); $body}
    };

    // for ($init; $cond; $step) { $body }
    ($init: stmt; $cond: expr; $step: expr; $body: block) => {
        {
            $init;
            while $cond {
                let mut _first = true;
                let mut _continue = false;
                // this loop runs once, allowing us to use `break` and
                // `continue` as `goto` to skip forward to the
                // condition.
                //
                // the booleans above are very transparent to the
                // optimiser, since they are modified exactly once,
                // with nice control flow, and this this optimises to
                // be similar to C for loop.
                loop {
                    // if we *don't* hit this, there was a `break` in
                    // the body (otherwise the loop fell-through or
                    // was `continue`d.)
                    if !_first { _continue = true; break }
                    _first = false;

                    $body
                }
                if !_continue {
                    // the `if` wasn't hit, so we should propagate the
                    // `break`.
                    break
                }

                $step
            }
        }
    };
}
