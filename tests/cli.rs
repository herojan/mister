extern crate assert_cmd;
extern crate predicates;
extern crate tempfile;

use std::process::Command;

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::prelude::CommandCargoExt;
use predicates::str::contains;
use std::error::Error as StdError;
use tempfile::TempDir;

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn StdError>> {
    let mut cmd = Command::main_binary()?;
    cmd.arg("-d").arg("test/file/doesnt/exist");
    cmd.assert()
        .failure()
        .stderr(contains("No such file or directory"));

    Ok(())
}

#[test]
fn mister_happy_path() -> Result<(), Box<dyn StdError>> {
    let output_dir = TempDir::new()?;
    let mut cmd = Command::main_binary()?;
    cmd.current_dir("tests");
    cmd.arg("-o")
        .arg(output_dir.path())
        .arg("-d")
        .arg("delivery.yaml")
        .arg("-b")
        .arg("mister-5");
    cmd.assert().success();

    Ok(())
}
