use std::collections::HashSet;
use std::hash::Hash;
use bit_set::BitSet;
use bracket_lib::prelude::VirtualKeyCode;
use bracket_terminal::prelude::Input;
use std::marker::PhantomData;

#[derive(Default)]
pub struct InputSnapshotState {
    pub pressed: BitSet,
    pub held: BitSet,
    pub released: BitSet,
}

pub trait InputSnapshot {
    type Input: Hash + Ord + Copy + Clone;

    fn key_to_usize(input: &Self::Input) -> usize;
    fn usize_to_key(at: usize) -> Self::Input;
    fn get_input_set(input: &Input) -> &HashSet<Self::Input>;

    fn get_snapshot(&self) -> &InputSnapshotState;
    fn get_snapshot_mut(&mut self) -> &mut InputSnapshotState;

    fn is_held(&self, key: Self::Input) -> bool {
        self.get_snapshot().held.contains(Self::key_to_usize(&key))
    }

    fn is_pressed(&self, key: Self::Input) -> bool {
        self.get_snapshot().pressed.contains(Self::key_to_usize(&key))
    }

    fn is_released(&self, key: Self::Input) -> bool {
        self.get_snapshot().released.contains(Self::key_to_usize(&key))
    }

    fn update(&mut self, input: &Input) {
        let snapshot = self.get_snapshot_mut();
        snapshot.pressed.clear();
        snapshot.released.clear();

        let current_press = Self::get_input_set(input);

        let mut unpressed = Vec::new();
        for index in &snapshot.held {
            let virtual_key = Self::usize_to_key(index);
            if !current_press.contains(&virtual_key) {
                snapshot.released.insert(index);
                unpressed.push(index);
            }
        }

        for index in unpressed {
            snapshot.held.remove(index);
        }

        for key in current_press {
            let index = Self::key_to_usize(key);

            if !snapshot.held.contains(index) {
                snapshot.pressed.insert(index);
            }

            snapshot.held.insert(index);
        }
    }
}

// keyboard

#[derive(Default)]
pub struct KeyboardSnapshot(InputSnapshotState);

impl InputSnapshot for KeyboardSnapshot {
    type Input = VirtualKeyCode;

    fn key_to_usize(input: &Self::Input) -> usize { *input as usize }

    fn usize_to_key(at: usize) -> Self::Input { KeyboardSnapshot::KEYS[at] }

    fn get_input_set(input: &Input) -> &HashSet<Self::Input> { &input.key_pressed_set() }

    fn get_snapshot(&self) -> &InputSnapshotState { &self.0 }

    fn get_snapshot_mut(&mut self) -> &mut InputSnapshotState { &mut self.0 }
}

impl KeyboardSnapshot {
    const KEYS: [VirtualKeyCode; 163] = [
        VirtualKeyCode::Key1,
        VirtualKeyCode::Key2,
        VirtualKeyCode::Key3,
        VirtualKeyCode::Key4,
        VirtualKeyCode::Key5,
        VirtualKeyCode::Key6,
        VirtualKeyCode::Key7,
        VirtualKeyCode::Key8,
        VirtualKeyCode::Key9,
        VirtualKeyCode::Key0,
        VirtualKeyCode::A,
        VirtualKeyCode::B,
        VirtualKeyCode::C,
        VirtualKeyCode::D,
        VirtualKeyCode::E,
        VirtualKeyCode::F,
        VirtualKeyCode::G,
        VirtualKeyCode::H,
        VirtualKeyCode::I,
        VirtualKeyCode::J,
        VirtualKeyCode::K,
        VirtualKeyCode::L,
        VirtualKeyCode::M,
        VirtualKeyCode::N,
        VirtualKeyCode::O,
        VirtualKeyCode::P,
        VirtualKeyCode::Q,
        VirtualKeyCode::R,
        VirtualKeyCode::S,
        VirtualKeyCode::T,
        VirtualKeyCode::U,
        VirtualKeyCode::V,
        VirtualKeyCode::W,
        VirtualKeyCode::X,
        VirtualKeyCode::Y,
        VirtualKeyCode::Z,
        VirtualKeyCode::Escape,
        VirtualKeyCode::F1,
        VirtualKeyCode::F2,
        VirtualKeyCode::F3,
        VirtualKeyCode::F4,
        VirtualKeyCode::F5,
        VirtualKeyCode::F6,
        VirtualKeyCode::F7,
        VirtualKeyCode::F8,
        VirtualKeyCode::F9,
        VirtualKeyCode::F10,
        VirtualKeyCode::F11,
        VirtualKeyCode::F12,
        VirtualKeyCode::F13,
        VirtualKeyCode::F14,
        VirtualKeyCode::F15,
        VirtualKeyCode::F16,
        VirtualKeyCode::F17,
        VirtualKeyCode::F18,
        VirtualKeyCode::F19,
        VirtualKeyCode::F20,
        VirtualKeyCode::F21,
        VirtualKeyCode::F22,
        VirtualKeyCode::F23,
        VirtualKeyCode::F24,
        VirtualKeyCode::Snapshot,
        VirtualKeyCode::Scroll,
        VirtualKeyCode::Pause,
        VirtualKeyCode::Insert,
        VirtualKeyCode::Home,
        VirtualKeyCode::Delete,
        VirtualKeyCode::End,
        VirtualKeyCode::PageDown,
        VirtualKeyCode::PageUp,
        VirtualKeyCode::Left,
        VirtualKeyCode::Up,
        VirtualKeyCode::Right,
        VirtualKeyCode::Down,
        VirtualKeyCode::Back,
        VirtualKeyCode::Return,
        VirtualKeyCode::Space,
        VirtualKeyCode::Compose,
        VirtualKeyCode::Caret,
        VirtualKeyCode::Numlock,
        VirtualKeyCode::Numpad0,
        VirtualKeyCode::Numpad1,
        VirtualKeyCode::Numpad2,
        VirtualKeyCode::Numpad3,
        VirtualKeyCode::Numpad4,
        VirtualKeyCode::Numpad5,
        VirtualKeyCode::Numpad6,
        VirtualKeyCode::Numpad7,
        VirtualKeyCode::Numpad8,
        VirtualKeyCode::Numpad9,
        VirtualKeyCode::NumpadAdd,
        VirtualKeyCode::NumpadDivide,
        VirtualKeyCode::NumpadDecimal,
        VirtualKeyCode::NumpadComma,
        VirtualKeyCode::NumpadEnter,
        VirtualKeyCode::NumpadEquals,
        VirtualKeyCode::NumpadMultiply,
        VirtualKeyCode::NumpadSubtract,
        VirtualKeyCode::AbntC1,
        VirtualKeyCode::AbntC2,
        VirtualKeyCode::Apostrophe,
        VirtualKeyCode::Apps,
        VirtualKeyCode::Asterisk,
        VirtualKeyCode::At,
        VirtualKeyCode::Ax,
        VirtualKeyCode::Backslash,
        VirtualKeyCode::Calculator,
        VirtualKeyCode::Capital,
        VirtualKeyCode::Colon,
        VirtualKeyCode::Comma,
        VirtualKeyCode::Convert,
        VirtualKeyCode::Equals,
        VirtualKeyCode::Grave,
        VirtualKeyCode::Kana,
        VirtualKeyCode::Kanji,
        VirtualKeyCode::LAlt,
        VirtualKeyCode::LBracket,
        VirtualKeyCode::LControl,
        VirtualKeyCode::LShift,
        VirtualKeyCode::LWin,
        VirtualKeyCode::Mail,
        VirtualKeyCode::MediaSelect,
        VirtualKeyCode::MediaStop,
        VirtualKeyCode::Minus,
        VirtualKeyCode::Mute,
        VirtualKeyCode::MyComputer,
        VirtualKeyCode::NavigateForward,
        VirtualKeyCode::NavigateBackward,
        VirtualKeyCode::NextTrack,
        VirtualKeyCode::NoConvert,
        VirtualKeyCode::OEM102,
        VirtualKeyCode::Period,
        VirtualKeyCode::PlayPause,
        VirtualKeyCode::Plus,
        VirtualKeyCode::Power,
        VirtualKeyCode::PrevTrack,
        VirtualKeyCode::RAlt,
        VirtualKeyCode::RBracket,
        VirtualKeyCode::RControl,
        VirtualKeyCode::RShift,
        VirtualKeyCode::RWin,
        VirtualKeyCode::Semicolon,
        VirtualKeyCode::Slash,
        VirtualKeyCode::Sleep,
        VirtualKeyCode::Stop,
        VirtualKeyCode::Sysrq,
        VirtualKeyCode::Tab,
        VirtualKeyCode::Underline,
        VirtualKeyCode::Unlabeled,
        VirtualKeyCode::VolumeDown,
        VirtualKeyCode::VolumeUp,
        VirtualKeyCode::Wake,
        VirtualKeyCode::WebBack,
        VirtualKeyCode::WebFavorites,
        VirtualKeyCode::WebForward,
        VirtualKeyCode::WebHome,
        VirtualKeyCode::WebRefresh,
        VirtualKeyCode::WebSearch,
        VirtualKeyCode::WebStop,
        VirtualKeyCode::Yen,
        VirtualKeyCode::Copy,
        VirtualKeyCode::Paste,
        VirtualKeyCode::Cut,
    ];
}

// mouse

#[derive(Default)]
pub struct MouseSnapshot(InputSnapshotState);

impl InputSnapshot for MouseSnapshot {
    type Input = usize;

    fn key_to_usize(input: &Self::Input) -> usize { *input }

    fn usize_to_key(at: usize) -> Self::Input { at }

    fn get_input_set(input: &Input) -> &HashSet<Self::Input> { &input.mouse_button_pressed_set() }

    fn get_snapshot(&self) -> &InputSnapshotState { &self.0 }

    fn get_snapshot_mut(&mut self) -> &mut InputSnapshotState { &mut self.0 }
}

// snapshots

#[derive(Default)]
pub struct InputSnapshots {
    pub keyboard: KeyboardSnapshot,
    pub mouse: MouseSnapshot,
}

impl InputSnapshots {
    pub fn update(&mut self, input: &Input) {
        self.keyboard.update(input);
        self.mouse.update(input);
    }
}
