use std::fs::{File, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::io::Write;
use std::path::Path;

const INIT_FILE: &[u8;220] = b"#! /bin/sh
#
# /init executable file in the initramfs
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

    file.write_all(INIT_FILE).expect("Could not write init file");
    file.set_permissions(Permissions::from_mode(0o755)).unwrap();
}
