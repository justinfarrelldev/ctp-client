use crate::config::GameInstallation;
use crate::ui::{InstallationUI, UIResult};
use imgui::*;
use imgui_sdl2_support::SdlPlatform;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::{EventPump, Sdl, VideoSubsystem};
use std::time::Instant;

pub struct App {
    _sdl_context: Sdl,
    canvas: Canvas<Window>,
    event_pump: EventPump,
    imgui_context: Context,
    platform: SdlPlatform,
    installation_ui: InstallationUI,
    selected_game: Option<GameInstallation>,
    running: bool,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("CTP Client", 800, 600)
            .position_centered()
            .resizable()
            .build()?;

        let mut canvas = window.into_canvas().build()?;
        let event_pump = sdl_context.event_pump()?;

        let mut imgui_context = Context::create();
        imgui_context.set_ini_filename(None);

        let platform = SdlPlatform::new(&mut imgui_context);

        let installation_ui = InstallationUI::new()?;

        Ok(Self {
            _sdl_context: sdl_context,
            canvas,
            event_pump,
            imgui_context,
            platform,
            installation_ui,
            selected_game: None,
            running: true,
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Starting CTP Client application loop");

        while self.running {
            let frame_start = Instant::now();

            self.handle_events();
            self.render()?;

            let frame_time = frame_start.elapsed();
            if frame_time.as_millis() < 16 {
                std::thread::sleep(std::time::Duration::from_millis(16) - frame_time);
            }
        }

        log::info!("CTP Client shutting down");
        Ok(())
    }

    fn handle_events(&mut self) {
        for event in self.event_pump.poll_iter() {
            self.platform.handle_event(&mut self.imgui_context, &event);

            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    self.running = false;
                }
                _ => {}
            }
        }
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.canvas.set_draw_color(Color::RGB(100, 100, 100));
        self.canvas.clear();

        self.platform
            .prepare_frame(&mut self.imgui_context, &self.event_pump);

        let ui = self.imgui_context.frame();

        match self.installation_ui.render(&ui) {
            UIResult::PlayGame(installation) => {
                self.start_game(installation)?;
            }
            UIResult::Exit => {
                self.running = false;
            }
            UIResult::None => {}
        }

        if let Some(ref game) = self.selected_game {
            self.render_game_screen(&ui, game);
        }

        self.platform
            .render(ui, self.canvas.window(), &mut self.canvas)?;

        self.canvas.present();
        Ok(())
    }

    fn start_game(
        &mut self,
        installation: GameInstallation,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!(
            "Starting game with installation: {}",
            installation.path.display()
        );
        self.selected_game = Some(installation);
        Ok(())
    }

    fn render_game_screen(&mut self, ui: &Ui, game: &GameInstallation) {
        ui.window("Game Screen")
            .size([400.0, 300.0], Condition::FirstUseEver)
            .position([200.0, 150.0], Condition::FirstUseEver)
            .build(|| {
                ui.text(&format!("Playing: {}", game.display_name));
                ui.text(&format!("Game Path: {}", game.path.display()));
                ui.text(&format!("Version: {:?}", game.version));
                ui.spacing();
                ui.text("Game implementation would go here!");
                ui.spacing();

                if ui.button("Back to Game Selection") {
                    self.selected_game = None;
                }
            });
    }
}
