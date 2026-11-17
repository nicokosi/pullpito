use std::env;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pullpito::run(env::args_os())
}
