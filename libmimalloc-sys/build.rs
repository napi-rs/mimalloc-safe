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

    if target_os == "windows" && target_env == "msvc" {
        build_mimalloc_win();
        return;
    }

    if env::var_os("CARGO_FEATURE_OVERRIDE").is_some() {
        cmake_config.define("MI_OVERRIDE", "ON");
    } else {
        cmake_config.define("MI_OVERRIDE", "OFF");
    }

    if env::var_os("CARGO_FEATURE_ASM").is_some() {
        cmake_config.define("MI_SEE_ASM", "ON");
    }

    if env::var_os("CARGO_FEATURE_SKIP_COLLECT_ON_EXIT").is_some() {
        cmake_config.define("MI_SKIP_COLLECT_ON_EXIT", "ON");
    }

    if env::var_os("CARGO_FEATURE_SECURE").is_some() {
        cmake_config.define("MI_SECURE", "ON");
        mimalloc_base_name = Cow::Owned(format!("{mimalloc_base_name}-secure"));
    }

    if env::var_os("CARGO_FEATURE_ETW").is_some() {
        cmake_config.define("MI_TRACK_ETW", "ON");
    }

    if env::var_os("CARGO_FEATURE_NO_OPT_ARCH").is_some() {
        cmake_config
            .define("MI_OPT_ARCH", "OFF")
            .define("MI_NO_OPT_ARCH", "ON");
    }

    if profile == "debug" {
        cmake_config
            .define("MI_DEBUG_FULL", "ON")
            .define("MI_SHOW_ERRORS", "ON");
        mimalloc_base_name = Cow::Owned(format!("{mimalloc_base_name}-debug"));
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

    let dst = cmake_config.build();

    println!("cargo:rustc-link-search=native={}/build", dst.display());

    println!("cargo:rustc-link-lib=static={mimalloc_base_name}");

    // on armv6 we need to link with libatomic
    if target_os == "linux" && target_arch == "arm" {
        // Embrace the atomic capability library across various platforms.
        // For instance, on certain platforms, llvm has relocated the atomic of the arm32 architecture to libclang_rt.builtins.a
        // while some use libatomic.a, and others use libatomic_ops.a.
        let atomic_name = env::var("DEP_ATOMIC").unwrap_or("atomic".to_owned());
        println!("cargo:rustc-link-lib={atomic_name}");
    }
}

fn build_mimalloc_win() {
    use std::env;

    let features = env::var("CARGO_CFG_TARGET_FEATURE")
        .map(|features| {
            features
                .split(",")
                .map(|f| f.to_owned())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    let mut build = cc::Build::new();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("target_arch not defined!");

    build
        .include("./c_src/mimalloc/include")
        .include("./c_src/mimalloc/src")
        .file("./c_src/mimalloc/src/static.c")
        .define("MI_BUILD_SHARED", "0")
        .cpp(false)
        .warnings(false)
        .flag_if_supported("-w");

    if env::var_os("CARGO_FEATURE_SECURE").is_some() {
        build.define("MI_SECURE", "4");
    }

    if env::var_os("CARGO_FEATURE_ASM").is_some() {
        build.flag_if_supported("-save-temps");
    }

    if env::var_os("CARGO_FEATURE_NO_OPT_ARCH").is_none() && target_arch == "arm64" {
        build.flag_if_supported("/arch:armv8.1");
    }

    if env::var_os("CARGO_FEATURE_SKIP_COLLECT_ON_EXIT").is_some() {
        build.define("MI_SKIP_COLLECT_ON_EXIT", "1");
    }

    // Remove heavy debug assertions etc
    let profile = std::env::var("PROFILE").unwrap();
    match profile.as_str() {
        "debug" => build.define("MI_DEBUG_FULL", "3"),
        "release" => build.define("MI_DEBUG_FULL", "0").define("MI_DEBUG", "0"),
        _ => build.define("MI_DEBUG_FULL", "3"),
    };

    if build.get_compiler().is_like_msvc() {
        build.cpp(true);
    }

    const LIBS: [&str; 5] = ["psapi", "shell32", "user32", "advapi32", "bcrypt"];

    for lib in LIBS {
        println!("cargo:rustc-link-lib={lib}");
    }

    if features.contains(&"crt-static".to_string()) {
        build.static_crt(true);
    }

    build.compile("mimalloc_safe_static");
}
