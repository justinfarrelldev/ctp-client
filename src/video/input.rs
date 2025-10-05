extern crate alloc;

/// Global keyboard and mouse input bridge for external video playback.
mod global;

use alloc::string::String;
use global::GlobalInputTracker;
use log::warn;
use sdl2::controller::{Button as ControllerButton, GameController};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::{EventPump, GameControllerSubsystem, JoystickSubsystem, Sdl};
use std::thread;
use std::time::Duration;

use super::types::SkipTrigger;

/// Collects skip-related input events on the main thread.
pub(super) struct SkipEventListener {
    /// SDL context backing the event pump; kept for subsystem lifetime management.
    _sdl: Sdl,
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
    /// Global keyboard and mouse input tracker for skip gestures.
    global_input: GlobalInputTracker,
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

        let mut global_input = GlobalInputTracker::with_default_provider();
        global_input.initialize_baseline();

        Ok(Self {
            _sdl: sdl,
            event_pump,
            controller_subsystem,
            _joystick_subsystem: joystick_subsystem,
            controllers,
            global_input,
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
                Self::apply_hotplug_action(
                    action,
                    &self.controller_subsystem,
                    &mut self.controllers,
                );
            }
        }

        if let Some(trigger) = self.global_input.poll_trigger() {
            return Some(trigger);
        }

        thread::sleep(Duration::from_millis(Self::POLL_INTERVAL_MS));
        None
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
