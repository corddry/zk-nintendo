//! A replay of NES state to be proven inside the zkVM.

// #![no_main]
// sp1_zkvm::entrypoint!(main);

use serde::{Deserialize, Serialize};
use tetanes::control_deck::ControlDeck;
use tetanes::input::{JoypadBtnState, Slot};
use tetanes::mem::RamState;

// Remove these
use image::ImageBuffer;
use std::path::Path;

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

const ROM: &[u8] = include_bytes!("../../../script/roms/Zelda.nes");
const REPLAY: &[u8] = include_bytes!("../../../script/replays/nes-input-history-zelda");
pub fn main() {
    // Moved here from script/src/main.rs
    let mut rom_bytes = ROM;
    let input_history: Vec<ControllerEvent> =
        bincode::deserialize::<Vec<ControllerEvent>>(REPLAY).expect("Failed to deserialize replay");

    // Read the rom and input history as array slices of bytes
    // let mut rom_bytes: &[u8] = &sp1_zkvm::io::read::<Vec<u8>>();
    // let input_history: Vec<ControllerEvent> = sp1_zkvm::io::read::<Vec<ControllerEvent>>();

    // Create a new NES control deck (the console itself)
    let mut control_deck = ControlDeck::new(RamState::AllZeros);

    // Load the rom into the control deck
    control_deck
        .load_rom("ROM", &mut rom_bytes)
        .expect("valid rom");

    // Copy the tetanes replay buffer technique from event.rs in lib.rs, then replay here
    // let decoded: Vec<ControllerEvent> = bincode::deserialize(input_history).unwrap(); // TODO: handle error

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
    control_deck
        .clock_frame()
        .expect("Invalid Opcode Encountered"); // Process last input -- needed?

    let a = control_deck.frame_number();
    let b = control_deck.frame_buffer().to_vec();
    let buf: ImageBuffer<image::Rgba<_>, Vec<u8>> =
        ImageBuffer::from_raw(256, 240, b).expect("Raw bytes to ImageBuffer failed");
    buf.save_with_format(Path::new("ProvenImage.png"), image::ImageFormat::Png)
        .expect("saving image failed");

    println!("a: {}", a);
    println!("b saved to ./ProvenImage.png");
}
