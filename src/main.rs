mod app;
mod components;
mod invoke;
mod notifications;

use app::*;
use leptos::*;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    log::info!("Hello, world!");

    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}
