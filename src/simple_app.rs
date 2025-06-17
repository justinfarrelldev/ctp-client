use crate::ui::{InstallationUI, UIResult};
use crate::config::GameInstallation;
use imgui::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{EventPump, Sdl};
use std::time::Instant;

pub struct SimpleApp {
    _sdl_context: Sdl,
    canvas: Canvas<Window>,
    event_pump: EventPump,
    installation_ui: InstallationUI,
    selected_game: Option<GameInstallation>,
    running: bool,
}

impl SimpleApp {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("CTP Client", 800, 600)
            .position_centered()
            .resizable()
            .build()?;

        let canvas = window.into_canvas().build()?;
        let event_pump = sdl_context.event_pump()?;
        let installation_ui = InstallationUI::new()?;

        Ok(Self {
            _sdl_context: sdl_context,
            canvas,
            event_pump,
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
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    self.running = false;
                }
                _ => {}
            }
        }
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.canvas.set_draw_color(Color::RGB(100, 100, 100));
        self.canvas.clear();

        // For now, just render basic info without ImGui
        self.canvas.present();
        Ok(())
    }

    fn start_game(&mut self, installation: GameInstallation) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Starting game with installation: {}", installation.path.display());
        self.selected_game = Some(installation);
        Ok(())
    }
}