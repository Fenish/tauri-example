fn main() {
    cc::Build::new()
        .cpp(true)
        .file("cpp/math.cpp")
        .compile("math");
        
    tauri_build::build()
}
