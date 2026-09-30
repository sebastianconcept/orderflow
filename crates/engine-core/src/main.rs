//! engine-core process entry.
//!
//! When this process starts, it constructs a Tokio Engine and exits.

use engine_core::run;

fn main() {
    let _engine = run();
    println!("engine-core program started");
}
