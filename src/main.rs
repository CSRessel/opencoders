mod app;
mod sdk;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    app::run()
}
