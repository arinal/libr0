//! Common utilities for exercises

/// Runs an exercise function, catching panics and printing results.
/// Returns Ok(()) if the test passes, Err(()) if it fails.
///
/// # Example
/// ```ignore
/// run!(my_test_function)?;
/// ```
#[macro_export]
macro_rules! run {
    ($func:ident) => {{
        use std::io::{stderr, Write};
        let name = stringify!($func).trim_start_matches('_');
        let name = name
            .trim_start_matches(|c: char| c.is_numeric())
            .trim_start_matches('_');
        print!("{}: ", name);
        std::io::stdout().flush().unwrap();

        // Temporarily suppress panic output
        let _guard = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $func()));

        // Restore panic hook
        std::panic::set_hook(_guard);

        match result {
            std::result::Result::Ok(_) => {
                println!("✓");
                std::result::Result::Ok(())
            }
            std::result::Result::Err(err) => {
                println!("✗ FAILED");
                if let std::option::Option::Some(msg) = err.downcast_ref::<&str>() {
                    eprintln!("  {}", msg);
                } else if let std::option::Option::Some(msg) = err.downcast_ref::<String>() {
                    eprintln!("  {}", msg);
                }
                std::result::Result::Err(())
            }
        }
    }};
}

/// Runs all listed exercises in sequence, stopping at first failure.
///
/// # Example
/// ```ignore
/// run_all!["Option0",
///     _01_test_one,
///     _02_test_two,
///     _03_test_three,
/// ];
/// ```
#[macro_export]
macro_rules! run_all {
    [$title:expr, $($func:ident),* $(,)?] => {{
        println!("=== {} Exercises ===\n", $title);

        let result: std::result::Result<(), ()> = (|| {
            $(
                run!($func)?;
            )*
            std::result::Result::Ok(())
        })();

        if result.is_ok() {
            println!("\n=== All {} tests passed! ===", $title);
        }

        result
    }};
}

fn main() {
    eprintln!("This is a utility module. Run exercises with:");
    eprintln!("  cargo run --example option");
    eprintln!("  cargo run --example result");
    eprintln!("  cargo run --example box");
}
