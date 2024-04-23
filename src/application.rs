use jandering_engine::{
    core::{
        bind_group::{camera::d2::D2CameraBindGroup, BindGroup},
        engine::{Engine, EngineContext},
        event_handler::EventHandler,
        renderer::{
            create_typed_bind_group, get_typed_bind_group, get_typed_bind_group_mut,
            BindGroupHandle, Renderer,
        },
        window::{InputState, Key, MouseButton, WindowEvent},
    },
    types::{Vec2, Vec3, DEG_TO_RAD},
};

use crate::{l_system::LSystem, shape_renderer::ShapeRenderer};

pub struct Application {
    last_time: web_time::Instant,
    time: f32,
    camera: BindGroupHandle<D2CameraBindGroup>,

    system: LSystem,

    shape_renderer: ShapeRenderer,
}

impl Application {
    pub async fn new(engine: &mut Engine) -> Self {
        let resolution = engine.renderer.size();
        let camera = create_typed_bind_group(
            engine.renderer.as_mut(),
            D2CameraBindGroup::new(resolution, true),
        );

        let shape_renderer = ShapeRenderer::new(engine.renderer.as_mut());

        let system = LSystem::new("".to_string(), 0);

        Self {
            last_time: web_time::Instant::now(),
            time: 0.0,
            camera,
            system,
            shape_renderer,
        }
    }

    fn draw_system(&mut self) {
        let branch_color = Vec3::new(0.5, 0.7, 0.1);
        let circle_color = Vec3::new(1.0, 0.5, 0.2);

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

        let mut states = vec![State::default()];

        for symbol in self.system.symbols() {
            match symbol {
                'F' => {
                    let state = states.last_mut().unwrap();
                    let end = state.position
                        + Vec2::from_angle(state.angle * DEG_TO_RAD).rotate(Vec2::new(0.0, 8.0));
                    self.shape_renderer
                        .draw_line(state.position, end, 1.0, branch_color);
                    state.position = end;
                }
                'A' => {
                    self.shape_renderer.draw_circle(
                        states.last().unwrap().position,
                        2.0,
                        circle_color,
                    );
                }
                '[' => states.push(states.last().unwrap().clone()),
                ']' => {
                    if states.len() > 1 {
                        states.pop();
                    } else {
                        states[0] = State::default()
                    }
                }
                '+' => {
                    states.last_mut().unwrap().angle += 15.0;
                }
                '-' => {
                    states.last_mut().unwrap().angle -= 15.0;
                }
                _ => {}
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
            self.system = LSystem::new("X".to_string(), 6);
        }

        self.draw_system();
    }
}

impl EventHandler for Application {
    fn on_update(&mut self, context: &mut EngineContext) {
        let current_time = web_time::Instant::now();
        let dt = (current_time - self.last_time).as_secs_f32();
        self.last_time = current_time;
        self.time += dt;

        let resolution = context.renderer.size();
        let camera = get_typed_bind_group_mut(context.renderer.as_mut(), self.camera).unwrap();
        camera.update(context.events, context.window, resolution, dt);

        if context.events.iter().any(|e| {
            matches!(
                e,
                WindowEvent::KeyInput {
                    key: Key::N,
                    state: InputState::Pressed
                }
            )
        }) {
            context.renderer.re_create_shaders();
        }

        self.run(context);

        self.shape_renderer.finish(context.renderer.as_mut());
    }

    fn on_render(&mut self, renderer: &mut Box<dyn Renderer>) {
        let camera = get_typed_bind_group(renderer.as_ref(), self.camera).unwrap();
        renderer.write_bind_group(self.camera.into(), &camera.get_data());

        let pass = renderer.new_pass().with_clear_color(0.05, 0.175, 0.25);
        self.shape_renderer.render(self.camera, pass).submit();
    }
}
