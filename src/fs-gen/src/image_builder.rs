//! Image builder module

use std::path::{Path, PathBuf};
use vfs::{FileSystem, OverlayFS, PhysicalFS, VfsPath};

/// Builds a new initramfs from path blobs and places it into a given destination folder
pub fn build_new_image(blob_paths: &Vec<PathBuf>, output_folder: &Path) {
    let virtual_paths = blob_paths
        .iter()
        .map(|p| VfsPath::new(PhysicalFS::new(p)))
        .collect::<Vec<VfsPath>>();

    let vfs = OverlayFS::new(&virtual_paths);
    let toto: Vec<String> = vfs.read_dir("/home").unwrap().collect();
    println!("{:?}", toto);
    println!("{:?}", vfs);
    let overlay_root: VfsPath = vfs.into();

    let output_vpath = VfsPath::new(PhysicalFS::new(output_folder));

    overlay_root
        .copy_dir(&output_vpath)
        .expect("Failed to copy the blobs !");
}
