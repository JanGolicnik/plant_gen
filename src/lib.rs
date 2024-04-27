use application::Application;
use jandering_engine::core::{engine::EngineBuilder, window::WindowBuilder};

mod application;
mod l_system;
mod shape_renderer;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
async fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Coultn init");

    let mut engine = EngineBuilder::new()
        .with_window(
            WindowBuilder::default()
                .with_cursor(true)
                .with_resolution(500, 500)
                .with_title("heyy")
                .with_cursor(true),
        )
        .build()
        .await;

    let application = Application::new(&mut engine).await;

    engine.run(application);
}
