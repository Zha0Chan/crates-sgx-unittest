#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(all(target_env = "sgx", target_vendor = "mesalock"), feature(rustc_private))]

#![feature(const_fn)]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

use std::string::String;
use std::vec::Vec;


pub use proc_macro::test_case;
#[cfg(feature = "default")]
pub use async_proc_macro::test;

pub struct TestCase(pub String, pub fn() -> ());


inventory::collect!(TestCase);

#[macro_export]
macro_rules! run_inventory_tests {
    ($predicate:expr) => {{
        crates_unittest::rsgx_unit_test_start();
        let mut ntestcases: u64 = 0u64;
        let mut failurecases: Vec<String> = Vec::new();
       
        for t in inventory::iter::<crates_unittest::TestCase>.into_iter() {
            
            if $predicate(&t.0) {
               
                crates_unittest::rsgx_unit_test(&mut ntestcases, &mut failurecases, t.1, &t.0);
            } 
        }

        crates_unittest::rsgx_unit_test_end(ntestcases, failurecases)
    }};
    () => {
        run_inventory_tests!(|_| true);
    };
}


/// This macro implements the fail test.
///
/// For example, in traditional Rust testing, we write
///
/// ```
/// #[test]
/// #[should_panic]
/// fn foo () {
///     assert!(false);
/// }
/// ```
///
/// This test would pass because it would panic and is expected to panic
/// (`#[should_panic]`).
///
/// An equivalent version of Rust SGX unit test is:
///
/// ```
/// fn foo() {
///     should_panic!(assert!(false));
/// }
/// ```
///
/// This requires developer to identify the line which triggers panic exactly.
#[macro_export]
macro_rules! should_panic {
    ($fmt:expr) => ({
        match ::std::panic::catch_unwind(|| { $fmt }).is_err() {
            true => {}
            false => { ::std::rt::begin_panic($fmt) }
        }
    });
}

/// This macro works as test case driver.
///
/// `rsgx_unit_tests!` works as a variadic function. It takes a list of test
/// case function as arguments and then execute them sequentially. It prints
/// the statistics on the test result at the end, and returns the amount of
/// failed tests. meaning if everything works the return vlaue will be 0.
///
/// One test fails if and only if it panics. For fail test (similar to
/// `#[should_panic]` in Rust, one should wrap the line which would panic with
/// macro `should_panic!`.
///
/// Here is one sample. For the entire sample, please reference to the sample
/// codes in this project.
///
/// ```
/// #[macro_use]
/// extern crate sgx_tstd as std;
/// #[macro_use]
/// extern crate sgx_unittest;
///
/// #[no_mangle]
/// pub extern "C"
/// fn test_ecall() -> sgx_status_t {
///     rsgx_unit_tests!(foo, bar, zoo);
///     sgx_status_t::SGX_SUCCESS
/// }
/// ```
#[macro_export]
macro_rules! rsgx_unit_tests {
    (
        $($f : expr),* $(,)?
    ) => {
        {
            rsgx_unit_test_start();
            let mut ntestcases : u64 = 0u64;
            let mut failurecases : Vec<String> = Vec::new();
            $(rsgx_unit_test(&mut ntestcases, &mut failurecases, $f,stringify!($f));)*
            rsgx_unit_test_end(ntestcases, failurecases)
        }
    }
}

/// A prologue function for Rust SGX unit testing.
///
/// To initiate the test environment, `rsgx_unit_tests!` macro would trigger
/// `rsgx_unit_test_start` at the very beginning. `rsgx_unit_test_start` inits
/// the test counter and fail test list, and print the prologue message.
pub fn rsgx_unit_test_start () {
    println!("\nstart running tests");
}

/// An epilogue function for Rust SGX unit testing.
///
/// `rsgx_unit_test_end` prints the statistics on test result, including
/// a list of failed tests and the statistics.
/// It will return the amount of failed tests. (success == 0)
pub fn rsgx_unit_test_end(ntestcases : u64, failurecases : Vec<String>) -> usize {
    let ntotal = ntestcases as usize;
    let nsucc  = ntestcases as usize - failurecases.len();

    if failurecases.len() != 0{
        print!("\nfailures: ");
        println!("    {}",
                 failurecases.iter()
                          .fold(
                              String::new(),
                              |s, per| s + "\n    " + per));
    }

    if ntotal == nsucc {
        print!("\ntest result \x1B[1;32mok\x1B[0m. ");
    } else {
        print!("\ntest result \x1B[1;31mFAILED\x1B[0m. ");
    }

    println!("{} tested, {} passed, {} failed", ntotal, nsucc, ntotal - nsucc);
    failurecases.len()
}

/// Perform one test case at a time.
///
/// This is the core function of sgx_tunittest. It runs one test case at a
/// time and saves the result. On test passes, it increases the passed counter
/// and on test fails, it records the failed test.
/// Required test function must be `Fn()`, taking nothing as input and returns
/// nothing.
pub fn rsgx_unit_test<F, R>(ncases: &mut u64, failurecases: &mut Vec<String>, f:F, name: &str)
    where F: FnOnce() -> R + std::panic::UnwindSafe {
    *ncases = *ncases + 1;
    match std::panic::catch_unwind (|| { f(); } ).is_ok() {
        true => {
            println!("{} {} ... {}!",
                    "testing",
                    name,
                    "\x1B[1;32mok\x1B[0m");
        }
        false => {
            println!("{} {} ... {}!",
                    "testing",
                    name,
                    "\x1B[1;31mfailed\x1B[0m");
            failurecases.push(String::from(name));
        }
    }
}

