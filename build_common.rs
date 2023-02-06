#[allow(dead_code)]
mod build {
    include!("build/conan_cargo_build.rs");
}

fn main() {
    for path in build::LIB_PATHS {
        // println!(r#"cargo:rustc-link-search=native={path}"#);

    }
    for lib in build::LIBS {
        println!(r#"cargo:rustc-link-lib={lib}"#);
    }
    for path in build::INCLUDE_PATHS {
        println!(r#"cargo:include={path}"#);
    }
}
