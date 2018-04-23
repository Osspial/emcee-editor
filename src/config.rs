use cgmath::Deg;
use derin::event::{Key, MouseButton};
use std::time::Duration;

pub type Keybind = Option<Key>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The mouse sensitivity, in degrees/pixel.
    pub mouse_sensitivity: Deg<f32>,
    pub camera_move_button: Option<MouseButton>,
    pub refresh_rate: Duration,
    pub keybindings: Keybindings
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybindings {
    pub move_forward: Keybind,
    pub move_backward: Keybind,
    pub move_left: Keybind,
    pub move_right: Keybind,
    pub move_up: Keybind,
    pub move_down: Keybind
}

impl Default for Config {
    fn default() -> Config {
        Config {
            mouse_sensitivity: Deg(0.5),
            camera_move_button: Some(MouseButton::Right),
            refresh_rate: Duration::new(1, 0) / 60,
            keybindings: Keybindings::default()
        }
    }
}

impl Default for Keybindings {
    fn default() -> Keybindings {
        use derin::event::Key::*;

        Keybindings {
            move_forward: Some(W),
            move_backward: Some(S),
            move_left: Some(A),
            move_right: Some(D),
            move_up: Some(LShift),
            move_down: Some(LCtrl)
        }
    }
}
