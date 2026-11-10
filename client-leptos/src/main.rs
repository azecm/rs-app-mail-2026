mod app;
mod components;
mod constants;
mod dom;
mod route;
mod store;
mod transport;
mod utils;

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::app);
}
