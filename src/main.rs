use application::Application;
use jandering_engine::core::{engine::EngineBuilder, window::WindowBuilder};

mod application;
mod l_system;
mod shape_renderer;

fn main() {
    let mut engine = EngineBuilder::default()
        .with_window(
            WindowBuilder::default()
                .with_cursor(true)
                .with_resolution(1000, 1000)
                .with_title("heyy")
                .with_cursor(true),
        )
        .build();

    let app = pollster::block_on(Application::new(&mut engine));

    engine.run(app);
}
