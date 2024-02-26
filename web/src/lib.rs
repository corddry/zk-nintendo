use tetanes::{
    audio::{AudioMixer, NesAudioCallback},
    control_deck::ControlDeck,
    input::{JoypadBtnState, Slot},
    mem::RamState,
    ppu::Ppu,
    video::VideoFilter,
};
use wasm_bindgen::prelude::*;

mod utils;

#[wasm_bindgen]
pub struct Nes {
    paused: bool,
    control_deck: ControlDeck,
    audio: AudioMixer,
    callback: NesAudioCallback,
    sound: bool,
    dynamic_rate_control: bool,
    dynamic_rate_delta: f32,
    controller_history: Vec<ControllerEvent>,
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
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

#[derive(Clone)]
#[wasm_bindgen]
pub struct ControllerEvent {
    pub btn: Button,
    pub pressed: bool,
    pub frame: u32,
}

#[wasm_bindgen]
impl Nes {
    pub fn init() {
        utils::set_panic_hook();
        utils::init_log();
    }

    pub fn new(output_sample_rate: f32, max_delta: f32) -> Self {
        let mut control_deck = ControlDeck::new(RamState::default());
        control_deck.set_filter(VideoFilter::Pixellate);
        let input_sample_rate = control_deck.sample_rate();
        let mut audio = AudioMixer::new(input_sample_rate, output_sample_rate, 4096);
        let callback = audio.open_callback().expect("valid callback");
        Self {
            paused: true,
            control_deck,
            audio,
            callback,
            sound: true,
            dynamic_rate_control: true,
            dynamic_rate_delta: max_delta,
            controller_history: Vec::new(),
        }
    }

    pub fn pause(&mut self, val: bool) {
        self.paused = val;
    }

    pub fn set_sound(&mut self, enabled: bool) {
        self.sound = enabled;
    }

    pub fn frame(&mut self) -> *const u8 {
        self.control_deck.frame_buffer().as_ptr()
    }

    pub fn audio_callback(&mut self, out: &mut [f32]) {
        self.callback.read(out);
    }

    pub fn width(&self) -> u32 {
        Ppu::WIDTH
    }

    pub fn height(&self) -> u32 {
        Ppu::HEIGHT
    }

    pub fn sample_rate(&self) -> f32 {
        self.audio.output_frequency()
    }

    pub fn clock_frame(&mut self) {
        self.control_deck.clock_frame().expect("valid clock");
        if self.sound {
            let samples = self.control_deck.audio_samples();
            self.audio
                .consume(samples, self.dynamic_rate_control, self.dynamic_rate_delta);
        }
        self.control_deck.clear_audio_samples();
    }

    pub fn load_rom(&mut self, mut bytes: &[u8]) {
        self.control_deck
            .load_rom("ROM", &mut bytes)
            .expect("valid rom");
        self.callback.clear();
    }

    pub fn controller_history(&self) -> Vec<ControllerEvent> {
        self.controller_history.clone()
    }

    fn record_controller_event(&mut self, btn: Button, pressed: bool) {
        let frame = self.control_deck.frame_number();
        self.controller_history.push(ControllerEvent {
            btn: btn,
            pressed,
            frame,
        });
    }

    pub fn handle_event(&mut self, key: &str, pressed: bool, repeat: bool) -> bool {
        if repeat {
            return false;
        }
        let joypad = &mut self.control_deck.joypad_mut(Slot::One);

        let btn: Option<Button> = match key {
            "Enter" => Some(Button::Start),
            "Shift" => Some(Button::Select),
            // "a" => Some(JoypadBtn::TURBO_A),
            // "s" => Some(JoypadBtn::TURBO_B),
            "z" => Some(Button::A),
            "x" => Some(Button::B),
            "ArrowUp" => Some(Button::Up),
            "ArrowDown" => Some(Button::Down),
            "ArrowLeft" => Some(Button::Left),
            "ArrowRight" => Some(Button::Right),
            _ => None,
        };

        if let Some(btn) = btn {
            joypad.set_button(JoypadBtnState::from(&btn), pressed);
            self.record_controller_event(btn, pressed);
            true
        } else {
            false
        }
    }
}

#[wasm_bindgen]
pub fn wasm_memory() -> JsValue {
    wasm_bindgen::memory()
}

impl Default for Nes {
    fn default() -> Self {
        Self::new(44_100.0, 0.005)
    }
}
