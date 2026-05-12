mod app;
mod history_page;
mod settings_page;
mod start_page;
mod toolbar;

fn main() {
    if let Err(error) = app::run() {
        eprintln!("failed to launch tartanos: {error:#}");
        std::process::exit(1);
    }
}
