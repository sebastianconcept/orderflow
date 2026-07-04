// Output sink – placeholder implementation

use engine_types::Execution;

pub struct OutputSink;

impl OutputSink {
    pub fn new() -> Self {
        Self
    }

    pub fn publish(&self, _execution: &Execution) {
        // TODO: real output implementation
    }
}
