use std::path::{Path, PathBuf};

use vfs::{FileSystem, OverlayFS, PhysicalFS, VfsPath};

pub fn build_new_image(blob_paths: &[PathBuf], output_folder: &Path) {
    let virtual_paths = blob_paths
        .iter()
        .map(|p| VfsPath::new(PhysicalFS::new(p)))
        .collect::<Vec<VfsPath>>();

    let vfs = OverlayFS::new(&virtual_paths);

    let toto: Vec<String> = vfs.read_dir("").unwrap().collect();
    // copy from the overlay fs to the output folder

    println!("{:?}", toto);
    println!("{:?}", vfs);
    let overlay_root: VfsPath = vfs.into();

    println!("{:?}", overlay_root);

    let output_vpath = VfsPath::new(PhysicalFS::new("."));
    println!("{:?}", output_vpath.as_str());

    overlay_root
        .join("bin")
        .unwrap()
        .copy_dir(&output_vpath.join("titi").unwrap())
        .expect("Failed to copy the blobs !");
}
