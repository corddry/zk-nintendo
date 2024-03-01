#![no_main]
sp1_zkvm::entrypoint!(main);

use serde::{Deserialize, Serialize};
use tetanes::control_deck::ControlDeck;
use tetanes::input::{JoypadBtnState, Slot};
use tetanes::mem::RamState;

//TODO: consider moving ControllerEvent and Button into their own module
#[derive(Serialize, Deserialize, Clone)]
pub struct ControllerEvent {
    pub btn: Button,
    pub pressed: bool,
    pub frame: u32,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Button {
    // Turbo disabled
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}
impl From<&Button> for JoypadBtnState {
    fn from(btn: &Button) -> JoypadBtnState {
        match btn {
            Button::A => JoypadBtnState::A,
            Button::B => JoypadBtnState::B,
            Button::Select => JoypadBtnState::SELECT,
            Button::Start => JoypadBtnState::START,
            Button::Up => JoypadBtnState::UP,
            Button::Down => JoypadBtnState::DOWN,
            Button::Left => JoypadBtnState::LEFT,
            Button::Right => JoypadBtnState::RIGHT,
        }
    }
}
pub fn main() {
    // Read the rom and input history as array slices of bytes
    let mut rom_bytes: &[u8] = &sp1_zkvm::io::read::<Vec<u8>>();
    let input_history: Vec<ControllerEvent> = sp1_zkvm::io::read::<Vec<ControllerEvent>>();

    // Create a new NES control deck (the console itself)
    let mut control_deck = ControlDeck::new(RamState::AllZeros);

    // Load the rom into the control deck
    control_deck
        .load_rom("ROM", &mut rom_bytes)
        .expect("valid rom");

    // Replay the input history by pressing buttons at the indicated frames
    for event in input_history {
        while (control_deck.frame_number()) < event.frame {
            control_deck
                .clock_frame()
                .expect("Invalid Opcode Encountered");
        }
        control_deck
            .joypad_mut(Slot::One)
            .set_button(JoypadBtnState::from(&event.btn), event.pressed);
    }

    // Advance the control deck to the last frame
    control_deck
        .clock_frame()
        .expect("Invalid Opcode Encountered"); // Process last input -- needed?

    // Write the frame number and frame buffer to the output
    sp1_zkvm::io::write(&control_deck.frame_number());
    sp1_zkvm::io::write::<Vec<u8>>(&control_deck.frame_buffer().to_vec());
}
