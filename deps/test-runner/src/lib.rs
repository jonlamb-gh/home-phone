//! Inspired by https://os.phil-opp.com/testing and https://github.com/japaric/utest

#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(crate::runner)]
#![reexport_test_harness_main = "test_main"]

use log::info;

pub fn runner(tests: &[&dyn Fn()]) {
    info!("running {} tests", tests.len());
    for test in tests {
        info!("test <no_name> ...");
        test();
        info!("\u{21b3} ok");
    }
    info!("test result: ok");
}
