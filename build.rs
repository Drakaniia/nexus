fn main() {
    slint_build::compile("ui/app_window.slint").unwrap();
    slint_build::compile("ui/setup_wizard.slint").unwrap();
    slint_build::compile("ui/app_settings.slint").unwrap();
}
