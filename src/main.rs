fn main() {
    let app = vaja_2::App::default();

    let native_options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(1280.0, 820.0)),
        min_window_size: Some(eframe::egui::vec2(900.0, 640.0)),
        ..Default::default()
    };
    eframe::run_native(Box::new(app), native_options);
}
