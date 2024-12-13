use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use std::{env, fs};

static MANIFEST_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    env::var("CARGO_MANIFEST_DIR")
        .expect("cargo to set \"CARGO_MANIFEST_DIR\" var")
        .into()
});
static OUT_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    env::var("OUT_DIR")
        .expect("cargo to set \"OUT_DIR\" var")
        .into()
});

static VENDOR_DIR: LazyLock<PathBuf> = LazyLock::new(|| MANIFEST_DIR.join("vendor"));
static WUFFS_SRC_DIR: LazyLock<PathBuf> = LazyLock::new(|| MANIFEST_DIR.join("src-wuffs"));

static WUFFS_C_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let dir = OUT_DIR.join("wuffs-c");

    if !dir.try_exists().unwrap() {
        fs::create_dir_all(&dir).unwrap();
    }

    dir
});
static WUFFS_OUT_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let dir = OUT_DIR.join("wuffs-out");

    if !dir.try_exists().unwrap() {
        fs::create_dir_all(&dir).unwrap();
    }

    dir
});

/// Tell cargo to rerun if any wuffs files changes
fn track_files() {
    for dir in [&VENDOR_DIR, &WUFFS_SRC_DIR] {
        for entry in fs::read_dir(dir.as_path())
            .unwrap_or_else(|_| panic!("cant read {} dir", dir.display()))
        {
            println!("cargo:rerun-if-changed={}", entry.unwrap().path().display());
        }
    }
}

/// Transpile everything in src-wuffs to C code
fn transpile_wuffs() {
    for result in fs::read_dir(WUFFS_SRC_DIR.as_path())
        .unwrap_or_else(|_| panic!("cant read {}", WUFFS_SRC_DIR.display()))
    {
        let entry = result.unwrap();

        let filename = PathBuf::from(entry.file_name());
        let outfile_path = WUFFS_C_DIR.join(format!("{}.c", filename.display()));
        let outfile = File::create(&outfile_path).expect("to create outfile");

        let mut child = Command::new("wuffs-c")
            .args(["gen", "-genlinenum", "-package_name", "ratmap"])
            .arg(entry.path())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to spawn wuffs-c");

        let stdout = child.stdout.take().expect("child stdout to be piped");
        let stderr = child.stderr.take().expect("child stderr to be piped");

        let mut reader = BufReader::new(stdout);
        let mut err_reader = BufReader::new(stderr);
        let mut writer = BufWriter::new(outfile);

        loop {
            let mut buffer = [0u8; 8192];
            match reader.read(&mut buffer) {
                Ok(0) => {
                    writer.flush().expect("to be able to flush outfile writer");
                    break;
                } // EOF reached
                Ok(n) => {
                    writer.write_all(&buffer[..n]).expect("to write to outfile");
                }
                Err(e) => panic!("Failed to read from stdout: {}", e),
            }
        }

        let mut err_output = String::new();
        err_reader
            .read_to_string(&mut err_output)
            .expect("to read from stderr");
        if !err_output.is_empty() {
            panic!("wuffs-c process errored:\n{err_output}");
        }

        if !child.wait().unwrap().success() {
            panic!("wuffs-c process exited with non-zero exit code");
        }
    }
}

/// Write `wuffs-base.c`
fn write_wuffs_base() {
    let wuffs_0_4 = File::open(VENDOR_DIR.join("wuffs-v0.4.c")).expect("to open wuffs-v0.4.c");
    let mut reader = BufReader::new(wuffs_0_4);

    let wuffs_base =
        File::create(WUFFS_C_DIR.join("wuffs-base.c")).expect("to create wuffs-base.c");
    let mut writer = BufWriter::new(wuffs_base);

    loop {
        let mut buffer = [0u8; 8192];
        match reader.read(&mut buffer) {
            Ok(0) => {
                writer.flush().expect("to flush outfile writer");
                break;
            } // EOF reached
            Ok(n) => {
                writer.write_all(&buffer[..n]).expect("to write to outfile");
            }
            Err(e) => panic!("Failed to read from infile: {}", e),
        }
    }
}

fn generate_bindings() {
    let cb = Box::new(bindgen::CargoCallbacks::new().rerun_on_header_files(false));
    let headers = fs::read_dir(WUFFS_C_DIR.as_path())
        .unwrap()
        .map(|result| result.unwrap().path().display().to_string());
    let outfile = WUFFS_OUT_DIR.join("bindings.rs");
    println!("cargo::rustc-env=BINDINGS_RS={}", outfile.display());

    let bindings = bindgen::Builder::default()
        .headers(headers)
        .clang_args([
            "-DWUFFS_IMPLEMENTATION",
            "-Wno-implicit-function-declaration",
        ])
        .parse_callbacks(cb)
        .use_core()
        .allowlist_var(".*wuffs.*")
        .allowlist_type(".*wuffs.*")
        .allowlist_function(".*wuffs.*")
        .allowlist_var(".*WUFFS.*")
        .allowlist_type(".*WUFFS.*")
        .allowlist_function(".*WUFFS.*")
        .allowlist_recursively(true)
        .generate()
        .expect("to generate bindings");

    bindings
        .write_to_file(&outfile)
        .expect("Failed to write bindings");
}

fn main() {
    // rerun-if-changed
    track_files();

    // wuffs -> c
    transpile_wuffs();
    write_wuffs_base();

    // c -> rs
    generate_bindings();
}
