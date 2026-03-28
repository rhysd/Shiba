use crate::persistent::DataDir;
use crate::renderer::{Event, RendererHandle};
use anyhow::{Context as _, Result};
use interprocess::local_socket::prelude::*;
use interprocess::local_socket::{GenericFilePath, ListenerOptions, Name, Stream};
use serde::{Deserialize, Serialize};
use std::fs::remove_file;
use std::io::{Read, Write};
use std::mem::take;
use std::path::PathBuf;
use std::thread;

#[derive(Serialize)]
struct Tx<'a> {
    init_files: &'a [PathBuf],
    watch_paths: &'a [PathBuf],
}

#[derive(Deserialize)]
struct Rx {
    init_files: Vec<PathBuf>,
    watch_paths: Vec<PathBuf>,
}

impl Rx {
    fn decode(stream: &mut Stream) -> Result<Self> {
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf)?;
        let len = u32::from_ne_bytes(buf) as usize;

        let mut buf = vec![0; len];
        stream.read_exact(&mut buf)?;
        let msg = serde_json::from_slice(&buf)?;

        Ok(msg)
    }
}

#[derive(Default)]
pub struct ProcessSingleton {
    name: Option<Name<'static>>,
    path: Option<PathBuf>,
}

impl ProcessSingleton {
    pub fn new(data_dir: &DataDir) -> Self {
        let Some(path) = data_dir.path() else {
            return Self::default();
        };
        let mut path = path.to_path_buf();
        path.push("singleton.sock");
        log::debug!("Socket file for IPC for process singleton: {path:?}");
        match path.clone().to_fs_name::<GenericFilePath>() {
            Ok(name) => Self { name: Some(name), path: Some(path) },
            Err(err) => {
                log::error!("Could not create a socket file {path:?} for IPC: {err:?}");
                Self::default()
            }
        }
    }

    pub fn send(&self, init_files: &[PathBuf], watch_paths: &[PathBuf]) -> Result<bool> {
        let Some(name) = &self.name else {
            log::debug!("Skip sending IPC message because socket file was not created");
            return Ok(false);
        };
        let mut conn = match Stream::connect(name.clone()) {
            Ok(conn) => conn,
            Err(err) => {
                log::debug!(
                    "Could not send IPC message probably because no process is running yet: {err:?}"
                );
                return Ok(false);
            }
        };
        let msg = serde_json::to_vec(&Tx { init_files, watch_paths })?;
        let len = u32::try_from(msg.len()).unwrap().to_ne_bytes();
        conn.write_all(&len)?;
        conn.write_all(&msg)?;
        conn.flush()?;
        Ok(true)
    }

    fn cleanup(&self) {
        if let Some(path) = &self.path {
            let _ = remove_file(path);
        }
    }

    pub fn listen<H: RendererHandle>(&mut self, handle: H) -> Result<()> {
        let Some(name) = take(&mut self.name) else {
            log::debug!("Skip listening IPC messages because socket file was not created");
            return Ok(());
        };

        // Socket file can be leaked when the previous app process was killed
        self.cleanup();

        let listener = ListenerOptions::new()
            .name(name)
            .create_sync()
            .context("Could not listen IPC messages")?;

        thread::spawn(move || {
            for conn in listener {
                match conn.context("Failed to listen IPC message") {
                    Ok(mut stream) => {
                        match Rx::decode(&mut stream).context("Failed to decode IPC message") {
                            Ok(Rx { init_files, watch_paths }) => {
                                let event = Event::ProcessSingleton { init_files, watch_paths };
                                handle.send(event);
                            }
                            Err(err) => handle.send(Event::Error(err)),
                        }
                    }
                    Err(err) => handle.send(Event::Error(err)),
                }
            }
        });

        Ok(())
    }
}

impl Drop for ProcessSingleton {
    fn drop(&mut self) {
        self.cleanup();
    }
}
