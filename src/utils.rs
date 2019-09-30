use std::time::Duration;

use nalgebra::Vector3;

/// Convert a Duration to a String formatted as HH:MM:SS
pub fn format_time(instant: Duration) -> String {
    let total_seconds = instant.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

/// Clamp a float between 0.0 and 255.0
///
/// This function is used due to an LLVM bug
/// where casting a float to u8 can lead to
/// undefined behavior:
/// https://github.com/rust-lang/rust/issues/10184
pub fn clamp(n: f32) -> f32 {
    n.min(255.0).max(0.0)
}


/// Tone map the given luminance globally with the Stockham equation
///
/// luminance is the pixel to map and max_luminance
/// is the smallest luminance that will be mapped to pure
/// white. Generally, this luminance is set to the
/// maximum luminance in the scene.
///
/// The tone mapping derivation can be found in Stockham's paper
/// Image Processing in the Context of a Visual Model.
pub fn stockham_tone_map(luminance: f32, max_luminance: f32) -> f32 {
    (luminance + 1.0).ln() / (max_luminance + 1.0).ln()
}

/// Tone map the given luminance globally with the Reinhard equation
///
/// luminance is the pixel to map and max_luminance
/// is the smallest luminance that will be mapped to pure
/// white. Generally, this luminance is set to the
/// maximum luminance in the scene.
///
/// The tone mapping derivation can be found in the paper:
/// Photographic Tone Reproduction for Digital Images by
/// Reinhard et al.
pub fn reinhard_tone_map(luminance: f32, max_luminance: f32) -> f32 {
    (luminance * (1.0 + (luminance / max_luminance.powf(2.0))))  / (1.0 + luminance)
}

/// Gamma correct the given luminance
pub fn gamma_correct(luminance: f32, gamma: f32) -> f32 {
    luminance.powf(1.0 / gamma)
}

/// Check if a computed color contains any NaNs
pub fn de_nan(color: &Vector3<f32>) -> Vector3<f32> {
    let mut correction = Vector3::new(color.x, color.y, color.z);
    (0..3).for_each(|i| {
              if correction[i].is_nan() {
                  correction[i] = 0.0
              }
          });
    correction
}
