use std::process::Command;
use std::io::{self, Write};
use glob::glob;

struct Cleaner
{}

impl Cleaner {
    pub fn new() -> Self {
        Cleaner{}
    }
}

impl Drop for Cleaner
{
    fn drop(&mut self) {
        let cleanup_output = Command::new("rm")
            .args(["-r", "target/tmp/system_interface", "tests/test_app/target"])
            .output()
            .expect("Failed to clean up");

        if ! cleanup_output.status.success()
        {
            eprintln!("Failed to clean up");
        }
    }
}

#[test]
fn generate_and_compile()
{
    let _cleaner = Cleaner::new();

    let mut compile_command = Command::new("target/debug/dbus-binding-rs");

    compile_command.args(["--output-dir", "target/tmp"]);

    for file in glob("xml/*/*.xml").expect("Failed to read directory")
    {
        compile_command.arg(file.unwrap().into_os_string());
    }

    let gen_output = compile_command.output()
        .expect("Failed to generate code");

    io::stdout().write_all(&gen_output.stdout).unwrap();
    io::stderr().write_all(&gen_output.stderr).unwrap();

    assert!(gen_output.status.success());

    let compile_output = Command::new("cargo")
        .arg("build")
        .current_dir("target/tmp/system_interface")
        .output()
        .expect("Failed to compile lib");

    io::stdout().write_all(&compile_output.stdout).unwrap();
    io::stderr().write_all(&compile_output.stderr).unwrap();

    assert!(compile_output.status.success());

    let compile_app_output = Command::new("cargo")
        .arg("build")
        .current_dir("tests/test_app")
        .output()
        .expect("Failed to compile test application");

    io::stdout().write_all(&compile_app_output.stdout).unwrap();
    io::stderr().write_all(&compile_app_output.stderr).unwrap();

    assert!(compile_app_output.status.success());
}
