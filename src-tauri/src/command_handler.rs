use tauri::Builder;

pub trait GenerateInvokeHandler {
	fn generate_invoke_handler(self) -> Builder<impl tauri::Runtime>;
}

impl<R: tauri::Runtime> GenerateInvokeHandler for Builder<R> {
	fn generate_invoke_handler(self) -> Builder<impl tauri::Runtime> {
		return self.invoke_handler(tauri::generate_handler![
			// CPP COMMANDS
			crate::cpp::calculator::calculate_sum,
			crate::cpp::calculator::calculate_multiply,
			crate::cpp::calculator::calculate_divide,
			crate::cpp::calculator::calculate_subtract,
			crate::cpp::calculator::run_stress_test,

			// RUST COMMANDS
			crate::rs::cms::rgb_to_cmyk,
			crate::rs::cms::rgb_to_lab,
		]);
	}
}
