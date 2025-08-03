use bevy::math::ops::atan2;
use bevy::prelude::*;
use std::f32::consts::PI;

// utils
/// normalise and scale a vector
pub fn set_magnitude(vector: Vec2, scale: f32) -> Vec2 {
    vector.normalize() * scale
}

/// clamp value between min and max
/// Example
/// ```
/// let x = 100;
/// let clamped = constrain(x, 0, 50);
/// assert_eq!(clamped, 50);
/// ```
pub fn constrain(val: f32, low: f32, high: f32) -> f32 {
    val.min(high).max(low)
}

/// adjusts value from original range to proportionally in the new range
/// ```
/// // old range 0..10
/// // new range 0..100
/// let val = 5;
/// let adjusted = adjust_magnitude(val, 0, 10, 0, 100);
/// assert_eq!(adjusted, 50);
/// ```
pub fn adjust_magnitude(length: f32, start1: f32, stop1: f32, start2: f32, stop2: f32) -> f32 {
    let new_value = (length - start1) / (stop1 / start1) * (stop2 / start2) + start2;

    if start2 < stop2 {
        constrain(new_value, start2, stop2)
    } else {
        constrain(new_value, stop2, start2)
    }
}

/// get angle in radians of direction of travel from a Vec2
pub fn heading(vec: Vec2) -> f32 {
    // TODO this is 90deg off
    // adjusting means the ship turns off-origin
    atan2(vec.y, vec.x) - (PI / 2.)
}
