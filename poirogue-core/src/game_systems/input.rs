use std::collections::VecDeque;
use bracket_lib::prelude::VirtualKeyCode;
use shipyard::{UniqueView, UniqueViewMut};
use crate::commands::{FlowCommand, GameCommand, GameplayContext};
use crate::game::FlagExit;
use bracket_lib::prelude::INPUT;
use crate::input::*;

pub fn make_input_snapshots(mut keyboard: UniqueViewMut<KeyboardSnapshot>,
                            mut mouse: UniqueViewMut<MouseSnapshot>) {
    use std::borrow::Borrow;
    keyboard.update(INPUT.lock().borrow());
    mouse.update(INPUT.lock().borrow());
}

pub fn on_input_keyboard_generate_level(keyboard: UniqueView<KeyboardSnapshot>,
                                        mut commands: UniqueViewMut<VecDeque<GameCommand>>,) {
    use GameCommand::*;
    use FlowCommand::*;

    if keyboard.is_pressed(VirtualKeyCode::F4) {
        commands.push_back(Flow(GenerateLevel));
    }
}

pub fn on_input_keyboard_exit(keyboard: UniqueView<KeyboardSnapshot>,
                              context: UniqueView<GameplayContext>,
                              mut exit: UniqueViewMut<FlagExit>,) {

    if *context == GameplayContext::MainGame &&
        keyboard.is_pressed(VirtualKeyCode::Escape) {

        exit.0 = true;
    }
}