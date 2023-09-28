use std::process::Command;
use std::io::{self, Write};
use glob::glob;
use fs_extra::dir::{CopyOptions, copy};

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
            .args(["-r", concat!(env!("CARGO_TARGET_TMPDIR"), "/test_app"), concat!(env!("CARGO_TARGET_TMPDIR"), "/system_interface")])
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

    let mut compile_command = Command::new(env!("CARGO_BIN_EXE_dbus-binding-rs"));

    compile_command.args(["--output-dir", env!("CARGO_TARGET_TMPDIR")]);

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
        .current_dir(concat!(env!("CARGO_TARGET_TMPDIR"), "/system_interface"))
        .output()
        .expect("Failed to compile lib");

    io::stdout().write_all(&compile_output.stdout).unwrap();
    io::stderr().write_all(&compile_output.stderr).unwrap();

    assert!(compile_output.status.success());

    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    copy("tests/test_app", env!("CARGO_TARGET_TMPDIR"), &copy_options).unwrap();

    let compile_app_output = Command::new("cargo")
        .arg("build")
        .current_dir(concat!(env!("CARGO_TARGET_TMPDIR"), "/test_app"))
        .output()
        .expect("Failed to compile test application");

    io::stdout().write_all(&compile_app_output.stdout).unwrap();
    io::stderr().write_all(&compile_app_output.stderr).unwrap();

    assert!(compile_app_output.status.success());
}
