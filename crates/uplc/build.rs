use std::{env, ffi::OsStr, fs, path::PathBuf};
use walkdir::WalkDir;

fn main() {
    // These tests currently fail because we do not support "counting mode" yet
    // Which means they will always run out of budget.
    // Once counting mode is implemented, these tests should not be skipped.
    let skip_tests = [
        "builtin_semantics_droplist_droplist_09",
        "builtin_semantics_droplist_droplist_10",
        "builtin_semantics_droplist_droplist_14",
        "builtin_semantics_droplist_droplist_15",
        "builtin_semantics_droplist_droplist_16",
    ];

    let crate_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let conformance_root = crate_root
        .parent()
        .unwrap()
        .join("uplc")
        .join("tests")
        .join("conformance");

    let flat_dir = conformance_root.join("flat");
    let textual_dir = conformance_root.join("textual");

    println!("cargo:rerun-if-changed={}", skip_tests.join(","));
    println!("cargo:rerun-if-changed={}", flat_dir.display());
    println!("cargo:rerun-if-changed={}", textual_dir.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let textual_tests = generate_textual_tests(&textual_dir, &skip_tests);
    fs::write(out_dir.join("generated_tests.rs"), textual_tests).unwrap();

    let flat_tests = generate_flat_tests(&flat_dir, &skip_tests);
    fs::write(out_dir.join("generated_flat_tests.rs"), flat_tests).unwrap();
}

fn generate_flat_tests(dir_path: &PathBuf, skip_tests: &[&str]) -> String {
    let mut tests = String::new();

    for entry in WalkDir::new(dir_path).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        if path.file_name().and_then(OsStr::to_str) != Some("fixture.json") {
            continue;
        }

        let test_name = path
            .strip_prefix(dir_path)
            .unwrap()
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(|c: char| !c.is_alphanumeric(), "_")
            .to_lowercase();

        let ignore = if skip_tests.contains(&test_name.as_str()) {
            "\n#[ignore]"
        } else {
            ""
        };

        let file_path = path.display();

        tests.push_str(&format!(
            r#"
{ignore}
#[test]
fn {test_name}() {{
    run_conformance(include_str!("{file_path}"));
}}
"#,
        ));
    }

    tests
}

fn generate_textual_tests(dir_path: &PathBuf, skip_tests: &[&str]) -> String {
    let mut tests = String::new();

    for entry in WalkDir::new(dir_path).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        if path.extension().and_then(OsStr::to_str) != Some("uplc") {
            continue;
        }

        let test_name = path
            .strip_prefix(dir_path)
            .unwrap()
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(|c: char| !c.is_alphanumeric(), "_")
            .to_lowercase();

        let ignore = if skip_tests.contains(&test_name.as_str()) {
            "\n#[ignore]"
        } else {
            ""
        };

        let file_path = path.display();
        let expected_path = path.with_extension("uplc.expected");
        let budget_path = path.with_extension("uplc.budget.expected");

        tests.push_str(&format!(
            r#"
{ignore}
#[test]
fn {test_name}() {{
    run_conformance(
        include_str!("{file_path}"),
        include_str!("{expected_path}"),
        include_str!("{budget_path}"),
    );
}}
"#,
            expected_path = expected_path.display(),
            budget_path = budget_path.display(),
        ));
    }

    tests
}
