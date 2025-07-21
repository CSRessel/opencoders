mod app;
mod sdk;

use app::App;

pub fn trim_lines_leading(s: &str) -> String {
    s.lines()
        .map(|line| line.trim_start().trim_end())
        .collect::<Vec<&str>>()
        .join("\n")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new()?;
    app.run()?;
    Ok(())
}
