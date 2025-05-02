use std::borrow::Cow;
use std::env;

use cmake::Config;

fn main() {
    let mut cmake_config = Config::new("c_src/mimalloc");

    let mut mimalloc_base_name = Cow::Borrowed("mimalloc");

    cmake_config
        .define("MI_BUILD_STATIC", "ON")
        .define("MI_BUILD_OBJECT", "OFF")
        .define("MI_BUILD_SHARED", "OFF")
        .define("MI_BUILD_TESTS", "OFF");

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("target_os not defined!");
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("target_arch not defined!");
    let target_env = env::var("CARGO_CFG_TARGET_ENV").expect("target_env not defined!");
    let profile = env::var("PROFILE").expect("profile not defined!");

    if env::var_os("CARGO_FEATURE_OVERRIDE").is_some() {
        cmake_config.define("MI_OVERRIDE", "ON");
    } else {
        cmake_config.define("MI_OVERRIDE", "OFF");
    }

    if env::var_os("CARGO_FEATURE_SKIP_COLLECT_ON_EXIT").is_some() {
        cmake_config.define("MI_SKIP_COLLECT_ON_EXIT", "ON");
    }

    if env::var_os("CARGO_FEATURE_SECURE").is_some() {
        cmake_config.define("MI_SECURE", "ON");
        mimalloc_base_name = Cow::Owned(format!("{}-secure", mimalloc_base_name));
    }

    if env::var_os("CARGO_FEATURE_ETW").is_some() {
        cmake_config.define("MI_TRACK_ETW", "ON");
    }

    if env::var_os("CARGO_FEATURE_NO_OPT_ARCH").is_some() {
        cmake_config.define("MI_OPT_ARCH", "OFF");
    }

    // it's complicated to link ucrt in debug mode on windows
    if profile == "debug" && target_env != "msvc" {
        cmake_config
            .define("MI_DEBUG_FULL", "ON")
            .define("MI_SHOW_ERRORS", "ON");
        mimalloc_base_name = Cow::Owned(format!("{}-debug", mimalloc_base_name));
    }

    if target_env == "musl" {
        cmake_config
            .define("MI_LIBC_MUSL", "ON")
            .cflag("-Wno-error=date-time");
        if target_arch == "aarch64" {
            cmake_config
                .define("MI_OPT_ARCH", "OFF")
                .define("MI_NO_OPT_ARCH", "ON");
        }
    }

    let dynamic_tls = env::var("CARGO_FEATURE_LOCAL_DYNAMIC_TLS").is_ok();

    if dynamic_tls {
        cmake_config.define("MI_LOCAL_DYNAMIC_TLS", "ON");
    }

    if (target_os == "linux" || target_os == "android")
        && env::var_os("CARGO_FEATURE_NO_THP").is_some()
    {
        cmake_config.define("MI_NO_THP", "1");
    }

    if env::var_os("CARGO_FEATURE_DEBUG").is_some()
        || (env::var_os("CARGO_FEATURE_DEBUG_IN_DEBUG").is_some() && cfg!(debug_assertions))
    {
        cmake_config.define("MI_DEBUG_FULL", "ON");
        cmake_config.define("MI_SHOW_ERRORS", "ON");
    } else {
        // Remove heavy debug assertions etc
        cmake_config.define("MI_DEBUG_FULL", "OFF");
    }

    if target_env == "msvc" {
        cmake_config
            .define("MI_USE_CXX", "ON")
            // always turn off debug full and show errors on msvc
            .define("MI_DEBUG_FULL", "OFF")
            .define("MI_SHOW_ERRORS", "OFF")
            .profile("release")
            .static_crt(false);
        // Link with libs needed on Windows
        // https://github.com/microsoft/mimalloc/blob/af21001f7a65eafb8fb16460b018ebf9d75e2ad8/CMakeLists.txt#L487
        // https://github.com/rust-lang/rust/issues/139352
        let libs = ["psapi", "shell32", "user32", "advapi32", "bcrypt"];

        for lib in libs {
            println!("cargo:rustc-link-lib={}", lib);
        }
    }

    let dst = cmake_config.build();

    if target_os == "windows" {
        println!(
            "cargo:rustc-link-search=native={}/build/Release",
            dst.display(),
        );
        println!("cargo:rustc-link-search=native={}/build", dst.display());
    } else {
        println!("cargo:rustc-link-search=native={}/build", dst.display());
    }

    println!("cargo:rustc-link-lib=static={}", mimalloc_base_name);

    // on armv6 we need to link with libatomic
    if target_os == "linux" && target_arch == "arm" {
        // Embrace the atomic capability library across various platforms.
        // For instance, on certain platforms, llvm has relocated the atomic of the arm32 architecture to libclang_rt.builtins.a
        // while some use libatomic.a, and others use libatomic_ops.a.
        let atomic_name = env::var("DEP_ATOMIC").unwrap_or("atomic".to_owned());
        println!("cargo:rustc-link-lib={}", atomic_name);
    }
}
