use std::panic;
use log::info;
use wasm_bindgen::prelude::wasm_bindgen;
use winit::event_loop::EventLoop;
use crate::app::state::AppState;

#[wasm_bindgen]
pub struct App {}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn create() -> Self {
        Self { }
    }

    #[wasm_bindgen]
    pub fn start(&self) {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Debug).expect("error initializing log");
        info!("Starting application...");
        let event_loop = EventLoop::new().unwrap();
        let _ = event_loop.run_app(&mut AppState::new());
    }
}