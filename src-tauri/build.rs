fn main() {
    cc::Build::new()
        .cpp(true)
        .file("cpp/math.cpp")
        .compile("math");
    
    println!("cargo:rustc-link-lib=static=math");
    println!("cargo:rerun-if-changed=cpp/math.cpp");
	println!("cargo:rustc-link-lib=lcms2");
        
    tauri_build::build()
}
