use bindgen::Formatter;
use std::fs;

fn generate_bindings() {
    fs::create_dir_all("gen").unwrap();

    bindgen::Builder::default()
        .header("vendor/slang/include/slang.h")
        .clang_arg("-I./vendor/slang")
        .clang_arg("-xc++")
        .clang_arg("-std=c++17")
        .allowlist_function("slang_.*")
        .allowlist_type("slang.*")
        .allowlist_type("FileSystemContentsCallBack")
        .allowlist_type("PathKind")
        .allowlist_var("SLANG_.*")
        .with_codegen_config(
            bindgen::CodegenConfig::FUNCTIONS
                | bindgen::CodegenConfig::TYPES
                | bindgen::CodegenConfig::VARS,
        )
        .layout_tests(false)
        .vtable_generation(true)
        .derive_copy(true)
        .formatter(Formatter::Rustfmt)
        .generate()
        .expect("Failed to generate bindings")
        .write_to_file("gen/bindings.rs")
        .expect("Failed to write bindings to file");
}

fn main() {
    generate_bindings();

    #[cfg(feature = "static")]
    {
        let mut cmake_config = cmake::Config::new("vendor/slang");
        cmake_config
            // The default 'install' target has way more stuff and is more
            // fragile to build. Only build the slang lib to save time
            // and increase chances of success.
            .build_target("slang")
            .define("SLANG_LIB_TYPE", "STATIC");

        let path = cmake_config.build();

        let cmake_build_type = cmake_config.get_profile();
        let lib_dir = path.join("build").join(cmake_build_type).join("lib");
        let lib_dir = lib_dir.to_str().unwrap();
        println!("cargo:rustc-link-search=native={lib_dir}");
        println!("cargo:rustc-link-lib=static=slang");
    }

    #[cfg(not(feature = "static"))]
    {
        #[cfg(any(target_os = "windows"))]
        {
            use std::env;
            use std::path::PathBuf;

            let slang_dir = env::var("SLANG_DIR").map(PathBuf::from).expect("Please provide an environment variable `SLANG_DIR` that points to your slang installation.");
            println!("cargo:rustc-link-search=native={}", slang_dir.join("bin").display());
            println!("cargo:rustc-link-search=native={}", slang_dir.join("lib").display());

            println!("cargo:rustc-link-lib=static=slang");
        }
        #[cfg(target_os = "linux")]
        println!("cargo:rustc-link-lib=slang");

        #[cfg(target_os = "macos")]
        println!("cargo:rustc-link-lib=dylib=slang");
    }
}
