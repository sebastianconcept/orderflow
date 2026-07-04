use engine_tokio::engine::TokioMatchingEngineBuilder;

fn main() {
    let _engine = TokioMatchingEngineBuilder::build();
    println!("engine-core program started");
}
