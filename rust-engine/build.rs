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
        .args([
            "--cudart",
            "shared",
            "-Xcompiler",
            "-fPIC",
            "-O3",
            "-DNDEBUG",
            "--use_fast_math",
            "-c",
            "src/cuda/rl_kernel.cu",
            "-o",
            &obj_path,
        ])
        .status();

    if let Ok(s) = status {
        if s.success() {
            let ar_status = Command::new("ar")
                .args(["crus", &lib_path, &obj_path])
                .status();

            if let Ok(ar_s) = ar_status {
                if ar_s.success() {
                    println!("cargo:rustc-link-search=native={}", out_dir);
                    println!("cargo:rustc-link-lib=static=rl_cuda");

                    // Tự động quét và liên kết các đường dẫn CUDA chuẩn trên Google Colab / Ubuntu / RHEL
                    let potential_cuda_paths = [
                        std::env::var("CUDA_HOME")
                            .map(|h| format!("{}/lib64", h))
                            .ok(),
                        std::env::var("CUDA_PATH")
                            .map(|p| format!("{}/lib64", p))
                            .ok(),
                        Some("/usr/local/cuda/lib64".to_string()),
                        Some("/usr/local/cuda/targets/x86_64-linux/lib".to_string()),
                        Some("/usr/local/cuda-12/lib64".to_string()),
                        Some("/usr/local/cuda-11/lib64".to_string()),
                        Some("/usr/lib/x86_64-linux-gnu".to_string()),
                        Some("/usr/lib64".to_string()),
                    ];

                    for dir in potential_cuda_paths.into_iter().flatten() {
                        if std::path::Path::new(&dir).exists() {
                            println!("cargo:rustc-link-search=native={}", dir);
                        }
                    }

                    println!("cargo:rustc-link-lib=dylib=cudart");
                    println!("cargo:rustc-link-lib=dylib=stdc++");
                    println!("cargo:rustc-cfg=has_cuda");
                }
            }
        }
    }
}
