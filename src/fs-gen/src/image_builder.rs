use std::{
    fs::{self},
    path::{Path, PathBuf},
    sync::Arc,
    thread,
};

use anyhow::{Context, Result};
use fuse_backend_rs::{
    api::{filesystem::Layer, server::Server},
    overlayfs::{config::Config, OverlayFs},
    passthrough::{self, PassthroughFs},
    transport::{FuseChannel, FuseSession},
};

pub struct FuseServer {
    server: Arc<Server<Arc<OverlayFs>>>,
    ch: FuseChannel,
}

type BoxedLayer = Box<dyn Layer<Inode = u64, Handle = u64> + Send + Sync>;

/// Initialiazes a passthrough fs for a given layer
/// a passthrough fs is just a dummy implementation to map to the physical disk
/// # Usage
/// ```
/// let passthrough_layer = new_passthroughfs_layer("/path/to/layer")
/// ```
fn new_passthroughfs_layer(rootdir: &str) -> Result<BoxedLayer> {
    let mut config = passthrough::Config::default();
    config.root_dir = String::from(rootdir);
    config.xattr = true;
    config.do_import = true;
    let fs = Box::new(PassthroughFs::<()>::new(config)?);
    fs.import()
        .with_context(|| format!("Failed to create the passthrough layer: {}", rootdir))?;
    Ok(fs as BoxedLayer)
}

/// Ensure a destination folder is created
fn ensure_folder_created(output_folder: &Path) -> Result<()> {
    fs::create_dir(output_folder).with_context(|| {
        format!(
            "Failed to ensure folder creation: {}",
            output_folder.to_string_lossy()
        )
    })
}

/// Merges all the layers into a single folder for further manipulation
/// It works by instantiating an overlay fs via FUSE then copying the files to the desired target
/// # Usage
/// ```
/// merge_layer(vec!["source/layer_1", "source/layer_2"], "/tmp/fused_layers")
/// ```
pub fn merge_layer(blob_paths: &[PathBuf], output_folder: &Path) -> Result<()> {
    // Stack all lower layers
    let mut lower_layers = Vec::new();
    for lower in blob_paths {
        lower_layers.push(Arc::new(new_passthroughfs_layer(&lower.to_string_lossy())?));
    }

    let mountpoint = Path::new("/tmp/cloudlet_internal");
    let fs_name = "cloudlet_overlay";

    ensure_folder_created(mountpoint)?;
    ensure_folder_created(output_folder)?;

    // Setup the overlay fs config
    let mut config = Config::default();
    config.work = "/work".into();
    config.mountpoint = output_folder.to_string_lossy().into();
    config.do_import = true;

    let fs = OverlayFs::new(None, lower_layers, config)
        .with_context(|| "Failed to construct the Overlay fs struct !".to_string())?;
    fs.import()
        .with_context(|| "Failed to initialize the overlay fs".to_string())?;

    // Enable a fuse session to make the fs available
    let mut se = FuseSession::new(mountpoint, fs_name, "", true)
        .with_context(|| "Failed to construct the Fuse session")?;
    se.set_allow_other(false);
    se.mount()
        .with_context(|| "Failed to mount the overlay fs".to_string())?;

    // Fuse session
    let mut server = FuseServer {
        server: Arc::new(Server::new(Arc::new(fs))),
        ch: se
            .new_channel()
            .with_context(|| "Failed to create a new channel".to_string())?,
    };

    let handle = thread::spawn(move || {
        let _ = server.svc_loop();
    });

    println!("copy starting !");
    //So now we need to copy the files
    dircpy::copy_dir(mountpoint, output_folder).with_context(|| {
        format!(
            "Failed to copy directories into the output folder: {}",
            output_folder.to_string_lossy()
        )
    })?;
    println!("copy finished");

    // Unmount sessions so it can be re-used in later executions of the program
    se.wake()
        .with_context(|| "Failed to exit the fuse session".to_string())?;
    se.umount()
        .with_context(|| "Failed to unmount the fuse session".to_string())?;

    let _ = handle.join();
    Ok(())
}

impl FuseServer {
    /// Run a loop to execute requests from the FUSE session
    ///
    pub fn svc_loop(&mut self) -> Result<()> {
        println!("entering server loop");
        loop {
            let value = self
                .ch
                .get_request()
                .with_context(|| "Failed to get message from fuse session".to_string())?;

            if value.is_none() {
                println!("fuse server exits");
                break;
            }

            // Technically the unwrap is safe
            let (reader, writer) = value.unwrap();

            if let Err(e) = self
                .server
                .handle_message(reader, writer.into(), None, None)
            {
                match e {
                    fuse_backend_rs::Error::EncodeMessage(_ebadf) => {
                        break;
                    }
                    _ => {
                        print!("Handling fuse message failed");
                        continue;
                    }
                }
            }
        }
        Ok(())
    }
}
