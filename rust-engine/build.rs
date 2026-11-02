use std::process::Command;

fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    println!("cargo:rustc-check-cfg=cfg(has_cuda)");
    if target_arch == "wasm32" {
        return;
    }

    println!("cargo:rerun-if-changed=src/cuda/rl_kernel.cu");

    // Biên dịch CUDA kernel nếu nvcc khả dụng
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let obj_path = format!("{}/rl_kernel.o", out_dir);
    let lib_path = format!("{}/librl_cuda.a", out_dir);

    let status = Command::new("nvcc")
        .args(&[
            "--cudart", "shared",
            "-Xcompiler", "-fPIC",
            "-c", "src/cuda/rl_kernel.cu",
            "-o", &obj_path,
        ])
        .status();

    if let Ok(s) = status {
        if s.success() {
            let ar_status = Command::new("ar")
                .args(&["crus", &lib_path, &obj_path])
                .status();

            if let Ok(ar_s) = ar_status {
                if ar_s.success() {
                    println!("cargo:rustc-link-search=native={}", out_dir);
                    println!("cargo:rustc-link-lib=static=rl_cuda");
                    println!("cargo:rustc-link-search=native=/usr/lib64");
                    println!("cargo:rustc-link-lib=dylib=cudart");
                    println!("cargo:rustc-link-lib=dylib=stdc++");
                    println!("cargo:rustc-cfg=has_cuda");
                }
            }
        }
    }
}
