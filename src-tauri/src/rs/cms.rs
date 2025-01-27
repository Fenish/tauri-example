extern "C" {
    fn cmsCreateTransform(
        input_profile: *mut std::ffi::c_void,
        input_format: u32,
        output_profile: *mut std::ffi::c_void,
        output_format: u32,
        intent: u32,
        flags: u32,
    ) -> *mut std::ffi::c_void;

    fn cmsDoTransform(
        transform: *mut std::ffi::c_void,
        input_buffer: *const f32,
        output_buffer: *mut f32,
        size: u32,
    );

    fn cmsCloseProfile(profile: *mut std::ffi::c_void);
    fn cmsDeleteTransform(transform: *mut std::ffi::c_void);
    fn cmsCreate_sRGBProfile() -> *mut std::ffi::c_void;  // Create default sRGB profile
}

// Color conversion functions
const TYPE_RGB_8: u32 = 0x02000000;
const TYPE_LAB_8: u32 = 0x03000000;
const INTENT_PERCEPTUAL: u32 = 0;

#[tauri::command]
pub fn rgb_to_lab(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    unsafe {
        // Use default sRGB profile instead of loading an ICC file
        let input_profile = cmsCreate_sRGBProfile();
        let output_profile = cmsCreate_sRGBProfile();  // You can simulate LAB profile like this

        let transform = cmsCreateTransform(
            input_profile,
            TYPE_RGB_8,
            output_profile,
            TYPE_LAB_8,
            INTENT_PERCEPTUAL,
            0,
        );

        let input = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];
        let mut output = [0.0f32; 3];

        cmsDoTransform(transform, input.as_ptr(), output.as_mut_ptr(), 1);

        // Close profiles and delete transform
        cmsCloseProfile(input_profile);
        cmsCloseProfile(output_profile);
        cmsDeleteTransform(transform);

        (output[0], output[1], output[2])  // Return the LAB values
    }
}

#[tauri::command]
pub fn rgb_to_cmyk(r: u8, g: u8, b: u8) -> (u8, u8, u8, u8) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let k = 1.0 - r.max(g).max(b);
    let c = (1.0 - r - k) / (1.0 - k).max(1.0);
    let m = (1.0 - g - k) / (1.0 - k).max(1.0);
    let y = (1.0 - b - k) / (1.0 - k).max(1.0);

    ((c * 255.0) as u8, (m * 255.0) as u8, (y * 255.0) as u8, (k * 255.0) as u8)
}
