use crate::persistent::DataDir;
use crate::renderer::{Event, RendererHandle};
use anyhow::{Context as _, Result};
use interprocess::local_socket::prelude::*;
use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, ListenerOptions, Name, Stream,
};
use serde::{Deserialize, Serialize};
use std::fs::remove_file;
use std::io::{Read, Write};
use std::mem::take;
use std::path::PathBuf;
use std::thread::{JoinHandle, spawn};

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
    fn decode(stream: &mut Stream) -> Result<Option<Self>> {
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf)?;
        let len = u32::from_ne_bytes(buf) as usize;

        if len == 0 {
            return Ok(None);
        }

        let mut buf = vec![0; len];
        stream.read_exact(&mut buf)?;
        let msg = serde_json::from_slice(&buf)?;

        Ok(Some(msg))
    }
}

#[derive(Default)]
pub struct ProcessSingleton {
    name: Option<Name<'static>>,
    path: Option<PathBuf>,
}

impl ProcessSingleton {
    #[allow(unused)]
    pub fn with_socket_file(data_dir: &DataDir) -> Self {
        let Some(path) = data_dir.path() else {
            return Self::default();
        };
        let path = path.join("singleton.sock");
        log::debug!("Socket file for IPC for process singleton: {path:?}");
        match path.clone().to_fs_name::<GenericFilePath>() {
            Ok(name) => Self { name: Some(name), path: Some(path) },
            Err(err) => {
                log::error!("Could not create a socket file {path:?} for IPC: {err:?}");
                Self::default()
            }
        }
    }

    #[allow(unused)]
    pub fn with_namespace() -> Self {
        match "shiba.singleton.sock".to_ns_name::<GenericNamespaced>() {
            Ok(name) => Self { name: Some(name), path: None },
            Err(err) => {
                log::error!("Could not create a socket namespace for IPC: {err:?}");
                Self::default()
            }
        }
    }

    fn stream(&self) -> Option<Stream> {
        let Some(name) = &self.name else {
            log::debug!("Skip sending IPC message because socket file was not created");
            return None;
        };
        match Stream::connect(name.clone()) {
            Ok(conn) => Some(conn),
            Err(err) => {
                log::debug!(
                    "Could not send IPC message probably because no process is running yet: {err:?}"
                );
                None
            }
        }
    }

    pub fn send(&self, init_files: &[PathBuf], watch_paths: &[PathBuf]) -> Result<bool> {
        let Some(mut conn) = self.stream() else {
            return Ok(false);
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

    pub fn listen<H: RendererHandle>(&mut self, handle: H) -> Result<Option<JoinHandle<()>>> {
        let Some(name) = take(&mut self.name) else {
            log::debug!("Skip listening IPC messages because socket file was not created");
            return Ok(None);
        };

        // Socket file can be leaked when the previous app process was killed
        self.cleanup();

        let listener = ListenerOptions::new()
            .name(name)
            .create_sync()
            .context("Could not listen IPC messages")?;

        let handle = spawn(move || {
            for conn in listener {
                match conn.context("Failed to listen IPC message") {
                    Ok(mut stream) => {
                        match Rx::decode(&mut stream).context("Failed to decode IPC message") {
                            Ok(Some(Rx { init_files, watch_paths })) => {
                                let event = Event::ProcessSingleton { init_files, watch_paths };
                                handle.send(event);
                            }
                            Ok(None) => break,
                            Err(err) => handle.send(Event::Error(err)),
                        }
                    }
                    Err(err) => handle.send(Event::Error(err)),
                }
            }
        });

        Ok(Some(handle))
    }

    pub fn can_listen(&self) -> bool {
        self.name.is_some()
    }

    #[allow(unused)]
    pub fn send_stop(&self) -> Result<bool> {
        let Some(mut conn) = self.stream() else {
            return Ok(false);
        };
        let len = 0u32.to_ne_bytes();
        conn.write_all(&len)?;
        conn.flush()?;
        Ok(true)
    }
}

impl Drop for ProcessSingleton {
    fn drop(&mut self) {
        self.cleanup();
    }
}

impl From<Name<'static>> for ProcessSingleton {
    fn from(name: Name<'static>) -> Self {
        Self { name: Some(name), path: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{Event, Renderer, Request};
    use crate::test::TestRenderer;
    use std::env;

    #[test]
    fn create_process_singleton() {
        let cwd = env::current_dir().unwrap();
        let singleton = ProcessSingleton::with_socket_file(&DataDir::new(cwd));
        assert!(singleton.can_listen());
        assert!(singleton.path.is_some());

        let singleton = ProcessSingleton::with_namespace();
        assert!(singleton.can_listen());
        assert!(singleton.path.is_none());

        let singleton = ProcessSingleton::default();
        assert!(!singleton.can_listen());
        assert!(singleton.path.is_none());
    }

    #[test]
    fn send_receive() {
        let renderer = TestRenderer::default();
        let dir = tempfile::tempdir().unwrap();

        let mut listener = ProcessSingleton::with_socket_file(&DataDir::new(dir.path()));
        let name = listener.name.clone().unwrap();

        let join_handle = listener.listen(renderer.create_handle()).unwrap().unwrap();
        assert!(!listener.can_listen());
        let path = listener.path.clone().unwrap();
        assert!(path.exists(), "{path:?}");

        let expected_init_files = &[PathBuf::from("foo.md")];
        let expected_watch_paths = &[PathBuf::from("some_dir")];
        let sender = ProcessSingleton::from(name);
        let sent = sender.send(expected_init_files, expected_watch_paths).unwrap();
        assert!(sent);

        let request = renderer.recv_timeout(1);
        assert!(
            matches!(
                &request,
                Request::Emit(
                    Event::ProcessSingleton {
                        init_files,
                        watch_paths,
                    }
                )
                if init_files == expected_init_files && watch_paths == expected_watch_paths
            ),
            "{request:?}"
        );

        let sent = sender.send_stop().unwrap();
        assert!(sent);

        join_handle.join().unwrap();

        // Check the socket file is cleaned up when the singleton is dropped
        drop(listener);
        assert!(!path.exists(), "{path:?}");
    }

    #[test]
    fn nop_empty_singleton() {
        let init_files = &[PathBuf::from("foo.md")];
        let watch_paths = &[PathBuf::from("some_dir")];
        let mut singleton = ProcessSingleton::default();
        let sent = singleton.send(init_files, watch_paths).unwrap();
        assert!(!sent);

        let renderer = TestRenderer::default();
        singleton.listen(renderer.create_handle()).unwrap();
    }
}
