// This file is responsible for calling the C++ functions that are defined in the math.cpp file.
extern "C" {
    fn sum(a: i32, b: i32) -> i32;
    fn multiply(a: i32, b: i32) -> i32;
    fn divide(a: i32, b: i32) -> i32;
    fn subtract(a: i32, b: i32) -> i32;
    fn stress_test(iterations: i32) -> i32;
}

#[tauri::command]
pub fn calculate_sum(a: i32, b: i32) -> i32 {
    unsafe { sum(a, b) }
}

#[tauri::command]
pub fn calculate_multiply(a: i32, b: i32) -> i32 {
    unsafe { multiply(a, b) }
}

#[tauri::command]
pub fn calculate_divide(a: i32, b: i32) -> i32 {
    unsafe { divide(a, b) }
}

#[tauri::command]
pub fn calculate_subtract(a: i32, b: i32) -> i32 {
    unsafe { subtract(a, b) }
}

#[tauri::command]
pub fn run_stress_test(iterations: i32) -> i32 {
    unsafe { stress_test(iterations) }
}
