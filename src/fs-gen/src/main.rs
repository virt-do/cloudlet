use std::{
    fs::{self, File},
    io::Result,
    path::{Path, PathBuf},
    process::exit,
    str::FromStr,
    sync::Arc,
    thread::{self, sleep},
    time::Duration,
};

use clap::Parser;
use fuse_backend_rs::{
    api::{filesystem::Layer, server::Server},
    overlayfs::{config::Config, OverlayFs},
    passthrough::{self, PassthroughFs},
    transport::{FuseChannel, FuseSession},
};

use crate::{cli_args::CliArgs, image_builder::build_new_image};
use signal_hook::{self, consts::TERM_SIGNALS, iterator::Signals};

mod cli_args;
mod image_builder;

pub struct FuseServer {
    server: Arc<Server<Arc<OverlayFs>>>,
    ch: FuseChannel,
}

type BoxedLayer = Box<dyn Layer<Inode = u64, Handle = u64> + Send + Sync>;

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

fn main() {
    // let args = CliArgs::get_args();
    // println!("Hello, world!, {:?}", args);

    // let paths: Vec<PathBuf> =
    //     vec![PathBuf::from_str("/home/spse/Downloads/image-gen/layer").unwrap()];

    // //let _ = fs::create_dir("./titi");

    // build_new_image(&paths, &PathBuf::from_str("./titi").unwrap());

    let upper_layer =
        Arc::new(new_passthroughfs_layer("/home/spse/Downloads/image-gen/layer_2").unwrap());
    let mut lower_layers = Vec::new();
    for lower in vec!["/home/spse/Downloads/image-gen/layer"] {
        lower_layers.push(Arc::new(new_passthroughfs_layer(&lower).unwrap()));
    }

    let mut config = Config::default();
    config.work = "/work".into();
    config.mountpoint = "/tmp/overlay_test2".into();
    config.do_import = true;

    print!("new overlay fs\n");
    let fs = OverlayFs::new(Some(upper_layer), lower_layers, config).unwrap();
    print!("init root inode\n");
    fs.import().unwrap();

    print!("open fuse session\n");
    let mut se =
        FuseSession::new(Path::new("/tmp/overlay_test2"), "toto_overlay2", "", false).unwrap();
    print!("session opened\n");
    se.mount().unwrap();

    let mut server = FuseServer {
        server: Arc::new(Server::new(Arc::new(fs))),
        ch: se.new_channel().unwrap(),
    };

    let handle = thread::spawn(move || {
        let _ = server.svc_loop();
    });

    // main thread
    let mut signals = Signals::new(TERM_SIGNALS).unwrap();
    for _sig in signals.forever() {
        break;
    }

    se.umount().unwrap();
    se.wake().unwrap();

    let _ = handle.join();
}

impl FuseServer {
    pub fn svc_loop(&mut self) -> Result<()> {
        print!("entering server loop\n");
        loop {
            if let Some((reader, writer)) = self.ch.get_request().unwrap() {
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
            } else {
                print!("fuse server exits");
                break;
            }
        }
        Ok(())
    }
}
