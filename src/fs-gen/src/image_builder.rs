use std::{
    fs::{self, File},
    io::Result,
    option,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
};

use fuse_backend_rs::{
    api::{filesystem::Layer, server::Server},
    overlayfs::{config::Config, OverlayFs},
    passthrough::{self, PassthroughFs},
    transport::{FuseChannel, FuseSession},
};
use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};

pub struct FuseServer {
    server: Arc<Server<Arc<OverlayFs>>>,
    ch: FuseChannel,
}

type BoxedLayer = Box<dyn Layer<Inode = u64, Handle = u64> + Send + Sync>;

/// Initialiazes a passthrough fs for a given layer
fn new_passthroughfs_layer(rootdir: &str) -> Result<BoxedLayer> {
    let mut config = passthrough::Config::default();
    config.root_dir = String::from(rootdir);
    // enable xattr
    config.xattr = true;
    config.do_import = true;
    let fs = Box::new(PassthroughFs::<()>::new(config)?);
    fs.import()?;
    Ok(fs as BoxedLayer)
}

/// Ensure a destination folder is created
fn ensure_folder_created(output_folder: &Path) -> Result<()> {
    //TODO if there is already a folder, change names/delete it beforehand ?
    let _ = fs::create_dir(output_folder);
    // TODO actually make sure it works
    Ok(())
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
        lower_layers.push(Arc::new(
            new_passthroughfs_layer(lower.to_str().unwrap()).unwrap(),
        ));
    }

    let mountpoint = Path::new("/tmp/cloudlet_internal");
    let fs_name = "cloudlet_overlay";

    let _ = ensure_folder_created(mountpoint);
    let _ = ensure_folder_created(output_folder);

    // Setup the overlay fs config
    let mut config = Config::default();
    config.work = "/work".into();
    config.mountpoint = output_folder.to_str().unwrap().into();
    config.do_import = true;

    let fs = OverlayFs::new(None, lower_layers, config).unwrap();
    fs.import().unwrap();

    // Enable a fuse session to make the fs available
    let mut se = FuseSession::new(mountpoint, fs_name, "", true).unwrap();
    se.set_allow_other(false);
    se.mount().unwrap();

    // Fuse session
    let mut server = FuseServer {
        server: Arc::new(Server::new(Arc::new(fs))),
        ch: se.new_channel().unwrap(),
    };

    let handle = thread::spawn(move || {
        let _ = server.svc_loop();
    });

    println!("copy starting !");
    //So now we need to copy the files
    let copy_res = dircpy::copy_dir(mountpoint, output_folder);
    println!("copy finished ?, {:?}", copy_res);

    // main thread
    // let mut signals = Signals::new(TERM_SIGNALS).unwrap();
    // for _sig in signals.forever() {
    //     break;
    // }

    // Unmount sessions so it can be re-used in later executions of the program
    se.umount().unwrap();
    se.wake().unwrap();

    let _ = handle.join();
    Ok(()) // TODO proper error handling
}

impl FuseServer {
    pub fn svc_loop(&mut self) -> Result<()> {
        print!("entering server loop\n");
        loop {
            match self.ch.get_request() {
                Ok(value) => {
                    if let Some((reader, writer)) = value {
                        if let Err(e) =
                            self.server
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
                    } else {
                        print!("fuse server exits");
                        break;
                    }
                }
                Err(err) => {
                    println!("{:?}", err);
                    break;
                }
            }
        }
        Ok(())
    }
}
