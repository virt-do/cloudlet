use std::fs::{File, Permissions, copy};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const INIT_FILE: &str = include_str!("../resources/initfile");

pub fn create_init_file(path: &Path, initfile: Option<PathBuf>) {
    let destination = path.join("init");

    if let Some(p) = initfile {
        // if there is a given initfile, we copy it into the folder
        copy(p, destination).expect("Could not copy initfile");
    } else {
        // if there is none, write the default init file
        let mut file = File::create(destination).unwrap();
        file.set_permissions(Permissions::from_mode(0o755)).unwrap();
        file.write_all(INIT_FILE.as_bytes())
            .expect("Could not write init file");
    }
}

pub fn generate_initramfs(root_directory: &Path, output: &Path) {
    let file = File::create(output).unwrap();
    file.set_permissions(Permissions::from_mode(0o644))
        .expect("Could not set permissions");

    println!("Generating initramfs...");

    let mut command = Command::new("sh")
        .current_dir(root_directory)
        .stdout(Stdio::from(file))
        .arg("-c")
        .arg("find . -print0 | cpio -0 --create --owner=root:root --format=newc | xz -9 --format=lzma")
        .spawn()
        .expect("Failed to package initramfs");
    command
        .wait()
        .expect("Failed to wait for initramfs to finish");

    println!("Initramfs generated!");
}
