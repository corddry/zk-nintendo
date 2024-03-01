use serde::{Deserialize, Serialize};
use sp1_core::{SP1Prover, SP1Stdin, SP1Verifier};
use std::path::Path;

use image::ImageBuffer;
// use std::env;

// TODO: make these CLI arguments (ignored now for simplicity)
const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");
const ROM: &[u8] = include_bytes!("../roms/Zelda.nes");
const REPLAY: &[u8] = include_bytes!("../replays/nes-input-history-zelda");

#[derive(Serialize, Deserialize, Clone, Debug)]
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
fn main() {
    sp1_core::utils::setup_logger();
    let mut stdin: SP1Stdin = SP1Stdin::new();

    // Convert the ROM and history files to vectors
    let rom_vec = ROM.to_vec();
    let history_vec: Vec<ControllerEvent> =
        bincode::deserialize::<Vec<ControllerEvent>>(REPLAY).expect("Failed to deserialize replay");

    stdin.write(&rom_vec);
    stdin.write(&history_vec);

    let mut proof = SP1Prover::prove(ELF, stdin).expect("proving failed");
    // // Read output.
    let a = proof.stdout.read::<u32>();
    let b = proof.stdout.read::<Vec<u8>>();

    // let mut stdout = SP1Prover::execute(ELF, stdin).expect("execution failed");
    // // Read output.
    // let a = stdout.read::<u32>();
    // let b = stdout.read::<Vec<u8>>();

    // Convert the frame data to a PNG and save it
    let buf: ImageBuffer<image::Rgba<_>, Vec<u8>> =
        ImageBuffer::from_raw(256, 240, b).expect("Raw bytes to ImageBuffer failed");
    buf.save_with_format(Path::new("ProvenImage"), image::ImageFormat::Png)
        .expect("saving image failed");

    println!("Frame Number: {}", a);
    println!("Frame saved to ./ProvenImage.png");

    // // Verify proof.
    SP1Verifier::verify(ELF, &proof).expect("verification failed");

    // Save proof.
    proof
        .save("proof-with-io.json")
        .expect("saving proof failed");

    println!("succesfully generated and verified proof for the program!")
}
