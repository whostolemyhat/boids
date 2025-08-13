# Boids

Steering behaviour implementation using Bevy, Avian2d and Rust.

Based on Coding Train/Nature of Code and Reynold's steering behaviours.

## Running

- `cargo run`

The main window shows buttons with options to change the steering behaviour. Most behaviours will target the mouse position (eg seek and arrive), and the ship is constrained to a smaller area in the middle of the screen.

## Structure

Most of the steering behaviour code is in `steering_plugin.rs`. The other files set up the window and handle things like moving the mouse around.
