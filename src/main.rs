use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, Region},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::wlr_layer::{
        Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
        LayerSurfaceConfigure,
    },
    shell::WaylandSurface,
    shm::{slot::SlotPool, Shm, ShmHandler},
};

use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};

struct App {
    registry_state: RegistryState,
    output_state: OutputState,
    compositor_state: CompositorState,
    shm: Shm,
    layer_shell: LayerShell,
    pool: Option<SlotPool>,
    width: u32,
    height: u32,
    layer_surface: Option<LayerSurface>,
    brightness: f32,
}

impl App {
    fn draw(&mut self, _qh: &QueueHandle<App>) {
        let width = self.width as i32;
        let height = self.height as i32;
        let stride = width * 4;

        let pool = self.pool.get_or_insert_with(|| {
            SlotPool::new(width as usize * height as usize * 4, &self.shm).unwrap()
        });

        if pool.len() < (width * height * 4) as usize {
             pool.resize((width * height * 4) as usize).expect("resize pool");
        }

        let (buffer, canvas) = pool
            .create_buffer(
                width,
                height,
                stride,
                wl_shm::Format::Argb8888,
            )
            .expect("create buffer");

        let alpha = (self.brightness * 255.0) as u8;
        for i in (0..canvas.len()).step_by(4) {
            canvas[i] = 0;
            canvas[i + 1] = 0;
            canvas[i + 2] = 0;
            canvas[i + 3] = alpha;
        }

        if let Some(surface) = &self.layer_surface {
            surface.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
            surface.wl_surface().damage(0, 0, width, height);
            
            let region = Region::new(&self.compositor_state).unwrap();
            surface.wl_surface().set_input_region(Some(region.wl_region()));
            
            surface.wl_surface().commit();
        }
    }
}

impl CompositorHandler for App {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {}

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {}

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {}
}

impl OutputHandler for App {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {}

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {}

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {}
}

impl LayerShellHandler for App {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        std::process::exit(0);
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        self.width = configure.new_size.0;
        self.height = configure.new_size.1;
        
        if self.width == 0 || self.height == 0 {
            return;
        }
        
        self.draw(qh);
    }
}

impl ShmHandler for App {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for App {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

delegate_compositor!(App);
delegate_output!(App);
delegate_shm!(App);
delegate_layer!(App);
delegate_registry!(App);

fn main() {
    let brightness = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse::<f32>().ok())
        .unwrap_or(0.3);

    let brightness = brightness.max(0.0).min(1.0);

    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init::<App>(&conn).unwrap();
    let qh = event_queue.handle();

    let registry_state = RegistryState::new(&globals);
    let output_state = OutputState::new(&globals, &qh);
    let compositor_state = CompositorState::bind(&globals, &qh).unwrap();
    let shm = Shm::bind(&globals, &qh).unwrap();
    let layer_shell = LayerShell::bind(&globals, &qh).unwrap();

    let mut app = App {
        registry_state,
        output_state,
        compositor_state,
        shm,
        layer_shell,
        pool: None,
        width: 0,
        height: 0,
        layer_surface: None,
        brightness,
    };

    let output = app
        .output_state
        .outputs()
        .next()
        .expect("No outputs found");

    let surface = app.compositor_state.create_surface(&qh);

    let layer = app.layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Overlay,
        Some("dark_overlay"),
        Some(&output), 
    );

    layer.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.set_exclusive_zone(-1);
    
    layer.commit();

    app.layer_surface = Some(layer);

    loop {
        event_queue.blocking_dispatch(&mut app).unwrap();
    }
}