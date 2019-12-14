#[cfg(feature = "denoise")]
/// Denoise the input buffer and return a denoised buffer
/// Reference: https://github.com/Twinklebear/oidn-rs/blob/master/examples/simple/src/main.rs
pub fn denoise(input: &Vec<u8>, width: usize, height: usize) -> Vec<u8> {
    // OIDN works on float images only, so convert this to a floating point image
    let coerced_input: Vec<f32> = input.iter().map(|&n| n as f32 / 255.0).collect();

    let mut filter_output = vec![0.0f32; coerced_input.len()];

    let mut device = oidn::Device::new();
    let mut filter = oidn::RayTracing::new(&mut device);
    filter.set_srgb(true).set_img_dims(width, height);
    filter.execute(&coerced_input[..], &mut filter_output[..]);

    if let Err(e) = device.get_error() {
        println!("Error denosing image: {}", e.1);
    }

    let output_buffer = filter_output.iter()
                                     .map(|&color| (255.0 * color) as u8)
                                     .collect();

    output_buffer
}
