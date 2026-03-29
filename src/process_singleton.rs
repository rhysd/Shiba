#[cfg(not(target_os = "windows"))]
use crate::persistent::DataDir;
use crate::renderer::{Event, RendererHandle};
use anyhow::{Context as _, Result};
#[cfg(not(target_os = "windows"))]
use interprocess::local_socket::GenericFilePath;
#[cfg(target_os = "windows")]
use interprocess::local_socket::GenericNamespaced;
use interprocess::local_socket::prelude::*;
use interprocess::local_socket::{ListenerOptions, Name, Stream};
use serde::{Deserialize, Serialize};
use std::fs::remove_file;
use std::io::{Read, Write};
use std::mem::take;
use std::path::PathBuf;
use std::thread::spawn;

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
    // `GenericFilePath` does not work on Windows due to 'not a named pipe path' error
    #[cfg(not(target_os = "windows"))]
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

    #[cfg(target_os = "windows")]
    pub fn with_namespace() -> Self {
        match "shiba.singleton.sock".to_ns_name::<GenericNamespaced>() {
            Ok(name) => Self { name: Some(name), path: None },
            Err(err) => {
                log::error!("Could not create a socket namespace for IPC: {err:?}");
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

        spawn(move || {
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

    pub fn can_listen(&self) -> bool {
        self.name.is_some()
    }
}

impl Drop for ProcessSingleton {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{Event, Renderer, Request};
    use crate::test::TestRenderer;

    #[test]
    fn create_process_singleton() {
        #[cfg(not(target_os = "windows"))]
        {
            let cwd = std::env::current_dir().unwrap();
            let singleton = ProcessSingleton::with_socket_file(&DataDir::new(cwd));
            assert!(singleton.can_listen());
            assert!(singleton.path.is_some());
        }

        #[cfg(target_os = "windows")]
        {
            let singleton = ProcessSingleton::with_namespace();
            assert!(singleton.can_listen());
            assert!(singleton.path.is_none());
        }

        let singleton = ProcessSingleton::default();
        assert!(!singleton.can_listen());
        assert!(singleton.path.is_none());
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn send_receive() {
        let renderer = TestRenderer::default();
        let dir = tempfile::tempdir().unwrap();

        let mut listener = ProcessSingleton::with_socket_file(&DataDir::new(dir.path()));
        assert!(listener.can_listen());

        listener.listen(renderer.create_handle()).unwrap();
        assert!(!listener.can_listen());
        let path = listener.path.clone().unwrap();
        assert!(path.exists(), "{path:?}");

        let expected_init_files = &[PathBuf::from("foo.md")];
        let expected_watch_paths = &[PathBuf::from("some_dir")];
        let sender = ProcessSingleton::with_socket_file(&DataDir::new(dir.path()));
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

        // Check the socket file is cleaned up when the singleton is dropped
        drop(listener);
        assert!(!path.exists(), "{path:?}");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn send_receive() {
        let renderer = TestRenderer::default();
        let mut listener = ProcessSingleton::with_namespace();
        assert!(listener.can_listen());

        listener.listen(renderer.create_handle()).unwrap();
        assert!(!listener.can_listen());

        let expected_init_files = &[PathBuf::from("foo.md")];
        let expected_watch_paths = &[PathBuf::from("some_dir")];
        let sender = ProcessSingleton::with_namespace();
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
