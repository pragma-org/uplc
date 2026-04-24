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

    let dir = "conformance"; // or whatever your previous macro argument was

    let dir_path = crate_root
        .parent()
        .unwrap()
        .join("uplc")
        .join("tests")
        .join(dir);

    println!("cargo:rerun-if-changed={}", skip_tests.join(","));
    println!("cargo:rerun-if-changed={}", dir_path.display());

    let mut tests = String::new();

    for entry in WalkDir::new(&dir_path).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        if path.extension().and_then(OsStr::to_str) != Some("uplc") {
            continue;
        }

        let test_name = path
            .strip_prefix(&dir_path)
            .unwrap()
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(|c: char| !c.is_alphanumeric(), "_")
            .to_lowercase();

        let file_path = path.display();
        let expected_path = path.with_extension("uplc.expected");
        let budget_path = path.with_extension("uplc.budget.expected");

        tests.push_str(&format!(
            r#"
{ignore}
#[test]
fn {test_name}() {{
    run_conformance_test(
        include_str!("{file_path}"),
        include_str!("{expected_path}"),
        include_str!("{budget_path}"),
    );
}}
"#,
            ignore = if skip_tests.contains(&test_name.as_str()) {
                "\n#[ignore]"
            } else {
                ""
            },
            expected_path = expected_path.display(),
            budget_path = budget_path.display(),
        ));
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("generated_tests.rs"), tests).unwrap();
}
