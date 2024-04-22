use std::fs::{File, Permissions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

const INIT_FILE: &[u8; 211] = b"#! /bin/sh
#
# Cloudlet initramfs generation
#
mount -t devtmpfs dev /dev
mount -t proc proc /proc
mount -t sysfs sysfs /sys
ip link set up dev lo

exec /sbin/getty -n -l /bin/sh 115200 /dev/console
poweroff -f
";

pub fn create_init_file(path: &Path) {
    let file_path = path.join("init");
    let mut file = File::create(file_path).unwrap();

    file.write_all(INIT_FILE)
        .expect("Could not write init file");
    file.set_permissions(Permissions::from_mode(0o755)).unwrap();
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
