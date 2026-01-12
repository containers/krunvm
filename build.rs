use std::path::Path;
use std::{env, fs, io, process};

const COMMANDS: [&str; 7] = [
    "krunvm",
    "krunvm-changevm",
    "krunvm-create",
    "krunvm-config",
    "krunvm-delete",
    "krunvm-list",
    "krunvm-start",
];

fn main() {
    let outdir = match env::var_os("OUT_DIR") {
        Some(outdir) => outdir,
        None => {
            panic!("OUT_DIR environment variable not defined.");
        }
    };
    fs::create_dir_all(&outdir).unwrap();

    for command in COMMANDS {
        if let Err(err) = generate_man_page(&outdir, command) {
            panic!("failed to generate man page: {}", err);
        }
    }

    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-search=/opt/homebrew/lib");
}

fn generate_man_page<P: AsRef<Path>>(outdir: P, command: &str) -> io::Result<()> {
    // If asciidoctor isn't installed, fallback to asciidoc.
    if let Err(err) = process::Command::new("asciidoctor").output() {
        eprintln!("Error from running 'asciidoctor': {}", err);
        return Err(err);
    }

    let outdir = outdir.as_ref();
    let outfile = outdir.join(format!("{}.1", command));
    let cwd = env::current_dir()?;
    let txt_path = cwd.join("docs").join(format!("{}.1.txt", command));

    let result = process::Command::new("asciidoctor")
        .arg("--doctype")
        .arg("manpage")
        .arg("--backend")
        .arg("manpage")
        .arg("--out-file")
        .arg(&outfile)
        .arg(&txt_path)
        .spawn()?
        .wait()?;
    if !result.success() {
        let msg = format!("'asciidoctor' failed with exit code {:?}", result.code());
        return Err(io::Error::other(msg));
    }
    Ok(())
}
