use anyhow::{Context, Result};
use std::fs::{copy as fscopy, File, Permissions};
use std::io::{copy as iocopy, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tracing::info;

const INIT_FILE: &str = include_str!("../resources/initfile");

pub fn create_init_file(path: &Path, initfile: Option<PathBuf>) -> Result<()> {
    info!("Writing initfile...");

    let destination = path.join("init");

    if let Some(p) = initfile {
        // if there is a given initfile, we copy it into the folder
        fscopy(p, destination)
            .with_context(|| "Failed to copy provided initfile to initramfs".to_string())?;
    } else {
        // if there is none, write the default init file
        let mut file = File::create(destination).unwrap();
        file.set_permissions(Permissions::from_mode(0o755)).unwrap();

        file.write_all(INIT_FILE.as_bytes())
            .with_context(|| "Failed to write default initfile to initramfs".to_string())?;
    }

    info!("Initfile written!");

    Ok(())
}

pub fn insert_agent(destination: &Path, agent_path: PathBuf) -> Result<()> {
    info!("Inserting agent into fs...");

    let mut file = File::create(destination.join("agent"))
        .with_context(|| "Could not open agent file inside initramfs".to_string())?;
    file.set_permissions(Permissions::from_mode(0o755))
        .with_context(|| "Failed to set permissions for agent file".to_string())?;

    let mut agent =
        File::open(agent_path).with_context(|| "Could not open host agent file".to_string())?;
    iocopy(&mut agent, &mut file)
        .with_context(|| "Failed to copy agent contents from host to destination".to_string())?;

    info!("Agent inserted!");

    Ok(())
}

pub fn generate_initramfs(root_directory: &Path, output: &Path) -> Result<()> {
    let file = File::create(output)
        .with_context(|| "Could not open output file to write initramfs".to_string())?;
    file.set_permissions(Permissions::from_mode(0o644))
        .with_context(|| "Failed to set permissions for output file".to_string())?;

    info!("Generating initramfs...");

    let mut command = Command::new("sh")
        .current_dir(root_directory)
        .stdout(Stdio::from(file))
        .arg("-c")
        .arg("find . -print0 | cpio -0 --create --owner=root:root --format=newc | xz -9 -T0 --format=lzma")
        .spawn()
        .with_context(|| "Failed to package initramfs into bundle".to_string())?;

    command.wait().with_context(|| {
        "Encountered exception while waiting for bundling to finish".to_string()
    })?;

    info!("Initramfs generated!");

    Ok(())
}
