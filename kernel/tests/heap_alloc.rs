#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_main]
extern crate alloc;
use alloc::vec::Vec;
use alloc::{boxed::Box, vec};
use bootloader::{BootInfo, entry_point};
use kernel::memory::global_alloc::HEAP_SIZE;

entry_point!(main);
fn main(boot_info: &'static BootInfo) -> ! {
    kernel::init(boot_info);
    kernel::enable_test();
    test_main();
    unreachable!()
}

#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[test_case]
fn many_boxes_long_lived() {
    let long_lived = Box::new(1);
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1);
}

#[test_case]
fn merge_small_allocs_to_large() {
    let small = [0u64; 2];
    let mut allocs = Vec::new();
    for _ in (0..HEAP_SIZE).step_by(16 * 2) {
        let x = Box::new(small);
        allocs.push(x);
    }
    core::mem::drop(allocs);
    let _large = vec![0u8; HEAP_SIZE - 1024];
}
