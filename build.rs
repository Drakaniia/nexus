fn main() {
    slint_build::compile("ui/main.slint").unwrap();
    // wizard.slint still needs a separate strategy if not imported
}
