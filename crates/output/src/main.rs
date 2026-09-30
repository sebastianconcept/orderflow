//! Output process entry.
//!
//! When this process starts, it constructs an OutputSink and exits.

use output::OutputSink;

fn main() {
    let sink = OutputSink::new();
    println!("output program started");
    let _ = sink;
}
