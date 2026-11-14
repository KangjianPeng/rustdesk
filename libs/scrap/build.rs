use std::{
    env, fs,
    path::{Path, PathBuf},
    println,
};

#[cfg(all(target_os = "linux", feature = "linux-pkg-config"))]
fn link_pkg_config(name: &str) -> Vec<PathBuf> {
    let pc_name = match name {
        "libvpx" => "vpx",
        _ => name,
    };
    match pkg_config::Config::new().statik(false).probe(pc_name) {
        Ok(lib) => lib.include_paths,
        // Debian/Ubuntu ship libyuv headers and a shared library without a .pc file.
        Err(pkg_config::Error::ProbeFailure { .. }) if name == "libyuv" => {
            println!("cargo:rustc-link-lib=dylib=yuv");
            Vec::new()
        }
        Err(error) => panic!(
            "unable to find {} development files: {}; install libvpx-dev, libaom-dev and libyuv-dev",
            pc_name, error
        ),
    }
}
#[cfg(not(all(target_os = "linux", feature = "linux-pkg-config")))]
fn link_pkg_config(_name: &str) -> Vec<PathBuf> {
    unimplemented!()
}

/// Link vcpkg package.
fn link_vcpkg(mut path: PathBuf, name: &str) -> PathBuf {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let mut target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_arch == "x86_64" {
        target_arch = "x64".to_owned();
    } else if target_arch == "x86" {
        target_arch = "x86".to_owned();
    } else if target_arch == "loongarch64" {
        target_arch = "loongarch64".to_owned();
    } else if target_arch == "aarch64" {
        target_arch = "arm64".to_owned();
    } else {
        target_arch = "arm".to_owned();
    }
    let mut target = if target_os == "macos" {
        if target_arch == "x64" {
            "x64-osx".to_owned()
        } else if target_arch == "arm64" {
            "arm64-osx".to_owned()
        } else {
            format!("{}-{}", target_arch, target_os)
        }
    } else if target_os == "windows" {
        format!("{}-windows-static", target_arch)
    } else {
        format!("{}-{}", target_arch, target_os)
    };
    if target_arch == "x86" {
        target = target.replace("x64", "x86");
    }
    println!("cargo:info={}", target);
    if let Ok(vcpkg_root) = std::env::var("VCPKG_INSTALLED_ROOT") {
        path = vcpkg_root.into();
    } else {
        path.push("installed");
    }
    path.push(target);
    println!(
        "cargo:rustc-link-lib=static={}",
        name.trim_start_matches("lib")
    );
    println!(
        "cargo:rustc-link-search={}",
        path.join("lib").to_str().unwrap()
    );
    let include = path.join("include");
    println!("cargo:include={}", include.to_str().unwrap());
    include
}

/// Link homebrew package(for Mac M1).
fn link_homebrew_m1(name: &str) -> PathBuf {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_os != "macos" || target_arch != "aarch64" {
        panic!("Couldn't find VCPKG_ROOT, also can't fallback to homebrew because it's only for macos aarch64.");
    }
    let mut path = PathBuf::from("/opt/homebrew/Cellar");
    path.push(name);
    let entries = if let Ok(dir) = std::fs::read_dir(&path) {
        dir
    } else {
        panic!("Could not find package in {}. Make sure your homebrew and package {} are all installed.", path.to_str().unwrap(),&name);
    };
    let mut directories = entries
        .into_iter()
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap().path())
        .filter(|x| x.is_dir())
        .collect::<Vec<_>>();
    // Find the newest version.
    directories.sort_unstable();
    if directories.is_empty() {
        panic!(
            "There's no installed version of {} in /opt/homebrew/Cellar",
            name
        );
    }
    path.push(directories.pop().unwrap());
    // Link the library.
    println!(
        "cargo:rustc-link-lib=static={}",
        name.trim_start_matches("lib")
    );
    // Add the library path.
    println!(
        "cargo:rustc-link-search={}",
        path.join("lib").to_str().unwrap()
    );
    // Add the include path.
    let include = path.join("include");
    println!("cargo:include={}", include.to_str().unwrap());
    include
}

/// Use system libraries on Linux with linux-pkg-config, otherwise vcpkg or Homebrew.
fn find_package(name: &str) -> Vec<PathBuf> {
    let no_pkg_config_var_name = format!("NO_PKG_CONFIG_{name}");
    println!("cargo:rerun-if-env-changed={no_pkg_config_var_name}");
    if cfg!(all(target_os = "linux", feature = "linux-pkg-config"))
        && std::env::var(no_pkg_config_var_name).as_deref() != Ok("1")
    {
        link_pkg_config(name)
    } else if let Ok(vcpkg_root) = std::env::var("VCPKG_ROOT") {
        vec![link_vcpkg(vcpkg_root.into(), name)]
    } else {
        // Try using homebrew
        vec![link_homebrew_m1(name)]
    }
}

fn generate_bindings(
    ffi_header: &Path,
    include_paths: &[PathBuf],
    ffi_rs: &Path,
    exact_file: &Path,
    regex: &str,
) {
    let mut b = bindgen::builder()
        .header(ffi_header.to_str().unwrap())
        .allowlist_type(regex)
        .allowlist_var(regex)
        .allowlist_function(regex)
        .rustified_enum(regex)
        .trust_clang_mangling(false)
        .layout_tests(false) // breaks 32/64-bit compat
        .generate_comments(false); // comments have prefix /*!\

    for dir in include_paths {
        b = b.clang_arg(format!("-I{}", dir.display()));
    }

    b.generate().unwrap().write_to_file(ffi_rs).unwrap();
    fs::copy(ffi_rs, exact_file).ok(); // ignore failure
}

fn gen_package(package: &str, ffi_header: &str, generated: &str, regex: &str) {
    let includes = find_package(package);
    let src_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = Path::new(&src_dir);
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    let ffi_header = src_dir.join("src").join("bindings").join(ffi_header);
    println!("rerun-if-changed={}", ffi_header.display());
    for dir in &includes {
        println!("rerun-if-changed={}", dir.display());
    }

    let ffi_rs = out_dir.join(generated);
    let exact_file = src_dir.join("generated").join(generated);
    generate_bindings(&ffi_header, &includes, &ffi_rs, &exact_file, regex);
}

fn main() {
    // in this crate, these are also valid configurations
    println!("cargo:rustc-check-cfg=cfg(dxgi,quartz,x11)");

    // there is problem with cfg(target_os) in build.rs, so use our workaround
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    env::remove_var("CARGO_CFG_TARGET_FEATURE");
    env::set_var("CARGO_CFG_TARGET_FEATURE", "crt-static");

    gen_package("libvpx", "vpx_ffi.h", "vpx_ffi.rs", "^[vV].*");
    gen_package("aom", "aom_ffi.h", "aom_ffi.rs", "^(aom|AOM|OBU|AV1).*");
    gen_package("libyuv", "yuv_ffi.h", "yuv_ffi.rs", ".*");

    if target_os == "ios" {
        // nothing
    } else if target_os == "android" {
        println!("cargo:rustc-cfg=android");
    } else if cfg!(windows) {
        // The first choice is Windows because DXGI is amazing.
        println!("cargo:rustc-cfg=dxgi");
    } else if cfg!(target_os = "macos") {
        // Quartz is second because macOS is the (annoying) exception.
        println!("cargo:rustc-cfg=quartz");
    } else if cfg!(unix) {
        // On UNIX we pray that X11 (with XCB) is available.
        println!("cargo:rustc-cfg=x11");
    }
}
