use std::{collections::HashMap, hash::Hash};

use async_trait::async_trait;
use jandering_engine::{
    core::{
        bind_group::{camera::d2::D2CameraBindGroup, BindGroup},
        engine::{Engine, EngineContext},
        event_handler::EventHandler,
        renderer::{
            create_typed_bind_group, get_typed_bind_group, get_typed_bind_group_mut,
            BindGroupHandle, Renderer,
        },
        window::{InputState, MouseButton, WindowEvent},
    },
    types::{Vec2, Vec3, DEG_TO_RAD},
};
use serde::Deserialize;
use wasm_bindgen::JsCast;

use crate::{
    l_system::{LSystem, LSystemConfig},
    shape_renderer::ShapeRenderer,
};

#[derive(Deserialize)]
enum Shape {
    Line {
        width: f32,
        length: f32,
        angle: f32,
        color: [f32; 3],
    },
    Circle {
        size: f32,
        color: [f32; 3],
    },
}

pub struct Application {
    last_time: web_time::Instant,
    time: f32,
    camera: BindGroupHandle<D2CameraBindGroup>,

    system: LSystem,

    shape_renderer: ShapeRenderer,

    should_redraw: bool,

    render_config: HashMap<char, Shape>,
}

impl Application {
    pub async fn new(engine: &mut Engine) -> Self {
        let resolution = engine.renderer.size();
        let mut camera = D2CameraBindGroup::new(resolution, true);
        camera.position.y = -500.0;
        let camera = create_typed_bind_group(engine.renderer.as_mut(), camera);

        let shape_renderer = ShapeRenderer::new(engine.renderer.as_mut()).await;

        let system = LSystem::new(LSystemConfig::default());

        Self {
            last_time: web_time::Instant::now(),
            time: 0.0,
            camera,
            system,
            shape_renderer,
            should_redraw: true,
            render_config: HashMap::new(),
        }
    }

    fn draw_system(&mut self) {
        #[derive(Clone)]
        struct State {
            position: Vec2,
            angle: f32,
        }

        impl Default for State {
            fn default() -> Self {
                Self {
                    position: Vec2::ZERO,
                    angle: 0.0,
                }
            }
        }

        let mut angle_change = 15.0;

        let mut states = vec![State::default()];
        for symbol in self.system.symbols().iter() {
            match symbol {
                '[' => states.push(states.last().unwrap().clone()),
                ']' => {
                    if states.len() > 1 {
                        states.pop();
                    } else {
                        states[0] = State::default()
                    }
                }
                '+' => {
                    states.last_mut().unwrap().angle += angle_change;
                }
                '-' => {
                    states.last_mut().unwrap().angle -= angle_change;
                }
                _ => {
                    if let Some(shape) = self.render_config.get(symbol) {
                        match shape {
                            Shape::Line {
                                width,
                                length,
                                angle,
                                color,
                            } => {
                                let state = states.last_mut().unwrap();
                                let end = state.position
                                    + Vec2::from_angle(state.angle * DEG_TO_RAD)
                                        .rotate(Vec2::new(0.0, *length));
                                self.shape_renderer.draw_line(
                                    state.position,
                                    end,
                                    *width,
                                    Vec3::from(*color),
                                );
                                state.position = end;
                                angle_change = *angle;
                            }
                            Shape::Circle { size, color } => {
                                self.shape_renderer.draw_circle(
                                    states.last().unwrap().position,
                                    *size,
                                    Vec3::from(*color),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn run(&mut self, context: &mut EngineContext) {
        if context.events.iter().any(|e| {
            matches!(
                e,
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state: InputState::Pressed
                }
            )
        }) {
            #[derive(Deserialize)]
            struct Config {
                shapes: HashMap<char, Shape>,
                rules: LSystemConfig,
            }

            let json = Self::get_rules_json();
            match serde_json::from_str::<Config>(&json) {
                Ok(config) => {
                    self.system = LSystem::new(config.rules);
                    self.render_config = config.shapes;
                    Self::output(self.system.symbols().iter().collect::<String>())
                }
                Err(e) => Self::output(e.to_string()),
            }
        }

        self.draw_system();
    }

    fn get_rules_json() -> String {
        let doc = web_sys::window().and_then(|win| win.document()).unwrap();
        let el = doc
            .get_element_by_id("lsystem_rules")
            .expect("should have a #lsystem_rules on the page");

        let textarea = el
            .dyn_ref::<web_sys::HtmlTextAreaElement>()
            .expect("#lsystem_rules should be an `HtmlTextAreaElement`");

        textarea.value()
    }

    fn output(text: String) {
        let doc = web_sys::window().and_then(|win| win.document()).unwrap();
        let el = doc
            .get_element_by_id("lsystem_output_textbox")
            .expect("should have a #lsystem_output_textbox on the page");
        el.set_inner_html(&text);
    }
}

#[async_trait]
impl EventHandler for Application {
    fn on_update(&mut self, context: &mut EngineContext<'_>) {
        let current_time = web_time::Instant::now();
        let dt = (current_time - self.last_time).as_secs_f32();
        self.last_time = current_time;
        self.time += dt;

        if context.events.iter().any(|e| {
            matches!(
                e,
                WindowEvent::MouseMotion(_)
                    | WindowEvent::MouseInput { .. }
                    | WindowEvent::Scroll(_)
            )
        }) {
            self.should_redraw = true;
        }

        let resolution = context.renderer.size();
        let camera = get_typed_bind_group_mut(context.renderer.as_mut(), self.camera).unwrap();
        camera.update(context.events, context.window, resolution, dt);
        let y_limit = -500.0 * camera.controller.as_ref().unwrap().zoom;
        if camera.position.y > y_limit {
            camera.position.y = y_limit;
            camera.update_data();
        }

        self.run(context);

        self.shape_renderer.finish(context.renderer.as_mut());
    }

    fn on_render(&mut self, renderer: &mut Box<dyn Renderer>) {
        if !self.should_redraw {
            return;
        }

        self.should_redraw = false;

        let camera = get_typed_bind_group(renderer.as_ref(), self.camera).unwrap();
        renderer.write_bind_group(self.camera.into(), &camera.get_data());

        let pass = renderer.new_pass().with_clear_color(0.05, 0.175, 0.25);
        self.shape_renderer.render(self.camera, pass).submit();
    }
}
