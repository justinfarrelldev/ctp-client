extern crate alloc;

use alloc::string::String;
use log::{debug, warn};
use sdl2::controller::{Button as ControllerButton, GameController};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{EventPump, GameControllerSubsystem, JoystickSubsystem, Sdl, VideoSubsystem};
use std::thread;
use std::time::Duration;

use super::overlay::SkipOverlayLayout;
use super::types::SkipTrigger;

/// Collects skip-related input events on the main thread.
pub(super) struct SkipEventListener {
    /// SDL context backing the event pump; kept for subsystem lifetime management.
    _sdl: Sdl,
    /// Video overlay canvas presenting skip instructions.
    canvas: Canvas<Window>,
    /// Event pump used to gather keyboard, mouse, and controller input.
    event_pump: EventPump,
    /// Controller subsystem leveraged for hot plug handling.
    controller_subsystem: GameControllerSubsystem,
    /// Joystick subsystem retained for the lifetime of controller enumeration.
    ///
    /// SDL requires the joystick subsystem to remain initialized while controllers are active;
    /// dropping it prematurely stops delivery of hot-plug events. Keeping the subsystem owned
    /// by the listener guarantees consistent accessibility support for controller users.
    _joystick_subsystem: JoystickSubsystem,
    /// Currently attached controllers opened for event delivery.
    controllers: Vec<GameController>,
}

impl SkipEventListener {
    /// Sleep interval (in milliseconds) between SDL event polls.
    const POLL_INTERVAL_MS: u64 = 16;

    /// Launches the event listener thread and waits for initialization to complete.
    ///
    /// # Errors
    ///
    /// Returns an error string if the thread cannot be spawned or SDL setup fails.
    pub(super) fn start() -> Result<Self, String> {
        let sdl = sdl2::init().map_err(|e| format!("Failed to initialize SDL: {e}"))?;
        let video = sdl
            .video()
            .map_err(|e| format!("Failed to initialize SDL video subsystem: {e}"))?;

        let mut canvas = Self::build_overlay_canvas(&video)?;
        Self::configure_overlay_canvas(&mut canvas);

        let (controller_subsystem, joystick_subsystem, mut controllers) =
            Self::prepare_controller_state(&sdl)?;

        let mut event_pump = sdl
            .event_pump()
            .map_err(|e| format!("Failed to acquire SDL event pump: {e}"))?;

        // Drain any startup events to avoid processing stale data on the first poll.
        for event in event_pump.poll_iter() {
            if let Some(action) = Self::interpret_hotplug_event(&event) {
                Self::apply_hotplug_action(action, &controller_subsystem, &mut controllers);
            }
        }

        Ok(Self {
            _sdl: sdl,
            canvas,
            event_pump,
            controller_subsystem,
            _joystick_subsystem: joystick_subsystem,
            controllers,
        })
    }

    /// Retrieves a pending skip trigger without blocking the caller.
    ///
    /// # Returns
    ///
    /// Returns `Some(SkipTrigger)` when an event is available, otherwise `None`.
    pub(super) fn poll_event(&mut self) -> Option<SkipTrigger> {
        for event in self.event_pump.poll_iter() {
            if let Some(trigger) = Self::derive_skip_trigger(&event) {
                return Some(trigger);
            }

            if let Some(action) = Self::interpret_hotplug_event(&event) {
                Self::apply_hotplug_action(action, &self.controller_subsystem, &mut self.controllers);
            }
        }

        self.canvas.present();
        thread::sleep(Duration::from_millis(Self::POLL_INTERVAL_MS));
        None
    }

    /// Builds the skip overlay window and returns a canvas for rendering.
    fn build_overlay_canvas(video: &VideoSubsystem) -> Result<Canvas<Window>, String> {
        let mut window = video
            .window(SkipOverlayLayout::window_title(), 640, 160)
            .position_centered()
            .allow_highdpi()
            .build()
            .map_err(|e| format!("Failed to create skip overlay window: {e}"))?;

        window.raise();
        window.set_always_on_top(true);

        window
            .into_canvas()
            .accelerated()
            .build()
            .map_err(|e| format!("Failed to build skip overlay canvas: {e}"))
    }

    /// Configures the overlay canvas with styling and interaction hints.
    fn configure_overlay_canvas(canvas: &mut Canvas<Window>) {
        if let Err(e) = canvas.window_mut().set_opacity(0.9) {
            debug!("Unable to adjust skip overlay opacity: {e}");
        }

        if SkipOverlayLayout::should_grab_input() {
            canvas.window_mut().set_mouse_grab(true);
            canvas.window_mut().set_keyboard_grab(true);
        }

        canvas.set_draw_color(SkipOverlayLayout::background_color());
        canvas.clear();
        if let Err(err) = SkipOverlayLayout::render_instructions(canvas) {
            warn!("Failed to render skip overlay instructions: {err}");
        }
        canvas.present();
    }

    /// Prepares controller subsystems and opens any already-attached controllers.
    fn prepare_controller_state(
        sdl: &Sdl,
    ) -> Result<
        (
            GameControllerSubsystem,
            JoystickSubsystem,
            Vec<GameController>,
        ),
        String,
    > {
        let controller_subsystem = sdl
            .game_controller()
            .map_err(|e| format!("Failed to initialize controller subsystem: {e}"))?;
        let joystick_subsystem = sdl
            .joystick()
            .map_err(|e| format!("Failed to initialize joystick subsystem: {e}"))?;

        let mut controllers: Vec<GameController> = Vec::new();
        let joystick_total = joystick_subsystem
            .num_joysticks()
            .map_err(|e| format!("Failed to enumerate joysticks: {e}"))?;
        for index in 0..joystick_total {
            Self::handle_controller_added(index, &controller_subsystem, &mut controllers);
        }

        Ok((controller_subsystem, joystick_subsystem, controllers))
    }

    /// Determines whether an SDL event should trigger a skip action.
    pub(super) const fn derive_skip_trigger(event: &Event) -> Option<SkipTrigger> {
        match event {
            &Event::Quit { .. }
            | &Event::MouseButtonDown {
                mouse_btn: MouseButton::Left | MouseButton::Right,
                ..
            } => Some(SkipTrigger::Mouse),
            &Event::KeyDown {
                keycode: Some(Keycode::Space | Keycode::Return | Keycode::KpEnter | Keycode::Escape),
                repeat: false,
                ..
            } => Some(SkipTrigger::Keyboard),
            &Event::ControllerButtonDown {
                button:
                    ControllerButton::A
                    | ControllerButton::B
                    | ControllerButton::X
                    | ControllerButton::Y
                    | ControllerButton::Start
                    | ControllerButton::Guide,
                ..
            } => Some(SkipTrigger::Controller),
            _ => None,
        }
    }

    /// Interprets controller hot-plug events into actionable instructions.
    /// Interprets controller hot-plug events into actionable instructions.
    pub(super) const fn interpret_hotplug_event(event: &Event) -> Option<HotplugAction> {
        match *event {
            Event::ControllerDeviceAdded { which, .. } => Some(HotplugAction::Add(which)),
            Event::ControllerDeviceRemoved { which, .. } => Some(HotplugAction::Remove(which)),
            _ => None,
        }
    }

    /// Applies a controller hot-plug action to the managed controller list.
    fn apply_hotplug_action(
        action: HotplugAction,
        controller_subsystem: &GameControllerSubsystem,
        controllers: &mut Vec<GameController>,
    ) {
        match action {
            HotplugAction::Add(which) => {
                Self::handle_controller_added(which, controller_subsystem, controllers);
            }
            HotplugAction::Remove(which) => {
                Self::handle_controller_removed(which, controllers);
            }
        }
    }

    /// Opens a controller reported by SDL, if it is compatible.
    pub(super) fn handle_controller_added(
        which: u32,
        controller_subsystem: &GameControllerSubsystem,
        controllers: &mut Vec<GameController>,
    ) {
        if controller_subsystem.is_game_controller(which) {
            match controller_subsystem.open(which) {
                Ok(controller) => controllers.push(controller),
                Err(err) => warn!("Failed to open controller {which}: {err}"),
            }
        }
    }

    /// Removes a controller that SDL reports as disconnected.
    pub(super) fn handle_controller_removed(which: u32, controllers: &mut Vec<GameController>) {
        controllers.retain(|controller| controller.instance_id() != which);
    }
}

/// Canvas and SDL resources naturally release on drop; no custom teardown necessary.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Actions describing how controller hot-plug updates should mutate the tracked controllers.
pub(super) enum HotplugAction {
    /// Opens a controller that SDL reports as newly connected.
    Add(u32),
    /// Removes a controller that SDL reports as disconnected.
    Remove(u32),
}
