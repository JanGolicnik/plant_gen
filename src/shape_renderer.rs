use jandering_engine::{
    core::{
        bind_group::camera::d2::D2CameraBindGroup,
        object::{D2Instance, Object, Vertex},
        renderer::{BindGroupHandle, RenderPass, Renderer, ShaderHandle},
        shader::{ShaderDescriptor, ShaderSource},
    },
    types::{Vec2, Vec3},
};

pub struct ShapeRenderer {
    line_shader: ShaderHandle,
    circle_shader: ShaderHandle,
    lines: Object<D2Instance>,
    circles: Object<D2Instance>,
    current_line: usize,
    current_circle: usize,
}

impl ShapeRenderer {
    pub async fn new(renderer: &mut dyn Renderer) -> Self {
        let line_shader = renderer.create_shader(
            ShaderDescriptor::flat()
                .with_descriptors(vec![Vertex::desc(), D2Instance::desc()])
                .with_bind_group_layouts(vec![D2CameraBindGroup::get_layout()])
                .with_backface_culling(false),
        );

        let circle_shader = renderer.create_shader(
            ShaderDescriptor::default()
                .with_source(ShaderSource::Code(include_str!(
                    "../res/shaders/circle_shader.wgsl"
                )))
                .with_descriptors(vec![Vertex::desc(), D2Instance::desc()])
                .with_bind_group_layouts(vec![D2CameraBindGroup::get_layout()])
                .with_backface_culling(false),
        );

        let lines = Object::quad(
            renderer,
            vec![D2Instance {
                scale: Vec2::new(100.0, 100.0),
                ..Default::default()
            }],
        );

        let circles = Object::triangle(
            renderer,
            vec![D2Instance {
                scale: Vec2::new(100.0, 100.0),
                ..Default::default()
            }],
        );

        Self {
            line_shader,
            circle_shader,
            lines,
            circles,
            current_line: 0,
            current_circle: 0,
        }
    }

    pub fn draw_line(&mut self, start: Vec2, end: Vec2, width: f32, color: Vec3) {
        let diff = end - start;
        let position = start + diff * 0.5;
        let instance = D2Instance {
            position,
            scale: Vec2::new(diff.length(), width),
            rotation: Vec2::X.angle_between(diff), //diff.angle_between(Vec2::X),
            color,
        };

        if self.lines.instances.len() <= self.current_line {
            self.lines.instances.push(instance);
        } else {
            self.lines.instances[self.current_line] = instance;
        }

        self.current_line += 1;
    }

    pub fn draw_circle(&mut self, position: Vec2, mut scale: f32, color: Vec3) {
        scale *= 2.0 / 3.0f32.sqrt();

        let instance = D2Instance {
            position,
            scale: Vec2::new(scale, scale),
            rotation: 0.0,
            color,
        };

        if self.circles.instances.len() <= self.current_circle {
            self.circles.instances.push(instance);
        } else {
            self.circles.instances[self.current_circle] = instance;
        }

        self.current_circle += 1;
    }

    pub fn finish(&mut self, renderer: &mut dyn Renderer) {
        self.lines.instances.truncate(self.current_line);
        self.current_line = 0;

        self.lines.update(renderer);

        self.circles.instances.truncate(self.current_circle);
        self.current_circle = 0;

        self.circles.update(renderer);
    }

    pub fn render<'renderer>(
        &mut self,
        camera: BindGroupHandle<D2CameraBindGroup>,
        render_pass: Box<dyn RenderPass<'renderer> + 'renderer>,
    ) -> Box<dyn RenderPass<'renderer> + 'renderer> {
        render_pass
            .set_shader(self.line_shader)
            .bind(0, camera.into())
            .render(&[&self.lines])
            .set_shader(self.circle_shader)
            .render(&[&self.circles])
    }
}
