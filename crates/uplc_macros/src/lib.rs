use std::{ffi::OsStr, fs, path::PathBuf};

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident};
use walkdir::WalkDir;

#[proc_macro]
pub fn generate_tests(input: TokenStream) -> TokenStream {
    let dir = parse_macro_input!(input as syn::LitStr);

    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir_path = crate_root
        .parent()
        .unwrap()
        .join("uplc")
        .join("tests")
        .join(dir.value());

    let mut test_functions = Vec::new();

    for entry in WalkDir::new(&dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.extension().and_then(OsStr::to_str) == Some("uplc") {
            let test_name = path
                .strip_prefix(&dir_path)
                .unwrap()
                .parent()
                .unwrap()
                .to_str()
                .unwrap()
                .replace(|c: char| !c.is_alphanumeric(), "_")
                .to_lowercase();

            let test_ident = Ident::new(&test_name, proc_macro2::Span::call_site());

            let file_contents = fs::read_to_string(path).expect("Failed to read file");

            let expected_contents = fs::read_to_string(path.with_extension("uplc.expected"))
                .expect("Failed to read file");

            let expected_budget = fs::read_to_string(path.with_extension("uplc.budget.expected"))
                .expect("Failed to read file");

            let test_fn = quote! {
                #[test]
                fn #test_ident() {
                    run_test(
                        #file_contents,
                        #expected_contents,
                        #expected_budget,
                    );
                }
            };

            test_functions.push(test_fn);
        }
    }

    let output = quote! {
        #(#test_functions)*
    };

    output.into()
}
