use winit::event_loop::EventLoop;
use wgpu_wasm::app::state::AppState;

pub fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut state = AppState::new();
    let _ = event_loop.run_app(&mut state);
}
