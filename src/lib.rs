//! Lunar Lander — Rust & WebAssembly arcade game.
//!
//! Exports a C-ABI interface for the browser host (`web/main.js`) to initialize
//! the simulation, feed keyboard/touch input, resize the viewport, and advance
//! frames at 60Hz.

pub mod camera;
pub mod collision;
pub mod ffi;
pub mod game;
pub mod hud;
pub mod lander;
pub mod lander_shape;
pub mod math;
pub mod terrain;
pub mod terrain_data;

use std::cell::RefCell;

use game::Game;
use hud::GameState;

thread_local! {
    static GAME: RefCell<Option<Game>> = const { RefCell::new(None) };
}

/// Initializes the global [`Game`] instance with the given canvas dimensions.
#[no_mangle]
pub extern "C" fn init_game(width: f64, height: f64) {
    GAME.with(|cell| {
        *cell.borrow_mut() = Some(Game::new(width, height));
    });
}

/// Advances the game simulation and renders one frame.
#[no_mangle]
pub extern "C" fn step_frame() {
    GAME.with(|cell| {
        if let Some(ref mut game) = *cell.borrow_mut() {
            game.step();
        }
    });
}

/// Updates player input flags (`1` = pressed, `0` = released).
#[no_mangle]
pub extern "C" fn set_input(left: u32, right: u32, thrust: u32) {
    GAME.with(|cell| {
        if let Some(ref mut game) = *cell.borrow_mut() {
            game.input_left = left != 0;
            game.input_right = right != 0;
            game.input_thrust = thrust != 0;
        }
    });
}

/// Starts a new game when in `Waiting` or `GameOver` state.
#[no_mangle]
pub extern "C" fn start_or_restart() {
    GAME.with(|cell| {
        if let Some(ref mut game) = *cell.borrow_mut() {
            if matches!(game.state, GameState::Waiting | GameState::GameOver) {
                game.start_new_game();
            }
        }
    });
}

/// Updates the viewport when the browser window resizes.
#[no_mangle]
pub extern "C" fn resize_viewport(width: f64, height: f64) {
    GAME.with(|cell| {
        if let Some(ref mut game) = *cell.borrow_mut() {
            game.view.resize(width, height, &mut game.lander);
        }
    });
}
