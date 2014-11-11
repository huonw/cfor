#![feature(phase)]

#[phase(plugin)] extern crate cfor;

#[test]
fn smoketest() {
    let mut tick = 0u;
    cfor!{let mut i = 1u; i <= 0b1000_0000; i *= 2 {
        tick += 1;
    }}

    assert_eq!(tick, 8);

    tick = 0;
    cfor!{let (mut a, mut b) = (0u, 0u); a + b < 20; { a += 1; b += 1; } {
        tick += 1;
    }}
    assert_eq!(tick, 10);
}

#[test]
#[should_fail]
fn continue_updates() {
    cfor!(let i = 0u; i < 10; panic!() {
        // we *should* hit the step expression.
        continue
    })
}

#[test]
fn missing_parts() {
    let mut inside = false;
    cfor!{;; () {
        inside = true;
        break
    }}
    assert!(inside);

    inside = false;
    cfor!{();; () {
        inside = true;
        break
    }}
    assert!(inside);

    cfor!{; false; () {
        panic!()
    }}
}
