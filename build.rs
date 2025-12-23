fn main() {
    slint_build::compile("ui/main.slint").unwrap();
    slint_build::compile("ui/wizard.slint").unwrap();
}
