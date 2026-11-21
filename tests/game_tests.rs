//! Integration tests for the Lunar Lander game state machine and physics.

use lunar_lander::game::Game;
use lunar_lander::hud::GameState;
use lunar_lander::lander::{Lander, GRAVITY};

#[test]
fn new_game_starts_in_playing_state_with_full_fuel() {
    let mut game = Game::new(1280.0, 720.0);
    assert_eq!(game.state, GameState::Waiting);
    game.start_new_game();
    assert_eq!(game.state, GameState::Playing);
    assert_eq!(game.lander.fuel, 1000.0);
    assert_eq!(game.score, 0);
}

#[test]
fn step_advances_frame_counter() {
    let mut game = Game::new(1280.0, 720.0);
    game.start_new_game();
    game.step();
    assert_eq!(game.counter, 1);
    assert_eq!(game.play_ticks, 1);
}

#[test]
fn landing_awards_multiplied_points_and_bonus_fuel() {
    let mut game = Game::new(1280.0, 720.0);
    game.start_new_game();
    game.handle_landed(5, true);
    assert_eq!((game.score, game.lander.fuel, game.state), (250, 1050.0, GameState::Landed));
}

#[test]
fn crash_with_no_fuel_triggers_game_over() {
    let mut game = Game::new(1280.0, 720.0);
    game.start_new_game();
    game.lander.fuel = 100.0;
    game.handle_crashed();
    assert_eq!((game.state, game.lander.fuel), (GameState::GameOver, 0.0));
}

#[test]
fn gravity_increases_downward_velocity() {
    let mut lander = Lander::new();
    let initial_vy = lander.vel.y;
    lander.update();
    assert!((lander.vel.y - (initial_vy + GRAVITY)).abs() < 1e-12);
}

#[test]
fn thrust_consumes_fuel_and_accelerates() {
    let mut lander = Lander::new();
    lander.rotation = 0.0;
    lander.target_rotation = 0.0;
    lander.set_thrust(1.0);
    for _ in 0..10 {
        lander.update();
    }
    assert!(lander.fuel < 1000.0);
    assert!(lander.vel.y < 0.0);
}
