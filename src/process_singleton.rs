use crate::cli::PathArgs;
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
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::fs::remove_file;
use std::io::{self, Read, Write};
use std::mem::take;
use std::path::PathBuf;
use std::thread::spawn;

fn encode<T: Serialize>(stream: &mut Stream, args: &T) -> io::Result<()> {
    let msg = serde_json::to_vec(args)?;
    let len =
        u32::try_from(msg.len()).expect("IPC message is smaller than 2^32 bytes").to_ne_bytes();
    stream.write_all(&len)?;
    stream.write_all(&msg)?;
    stream.flush()?;
    Ok(())
}

fn decode<T: DeserializeOwned>(stream: &mut Stream) -> io::Result<T> {
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf)?;
    let len = u32::from_ne_bytes(buf) as usize;

    let mut buf = vec![0; len];
    stream.read_exact(&mut buf)?;
    let msg = serde_json::from_slice(&buf)?;
    Ok(msg)
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

    pub fn send(&self, args: &PathArgs) -> Result<bool> {
        let Some(name) = &self.name else {
            log::debug!("Skip sending IPC message because socket file was not created");
            return Ok(false);
        };
        match Stream::connect(name.clone()) {
            Ok(mut conn) => {
                encode(&mut conn, args)?;
                log::debug!("Arguments are sent to the existing process singleton: {args:?}");
                Ok(true)
            }
            Err(err) => {
                log::debug!(
                    "Could not send IPC message probably because no server is running yet: {err:?}"
                );
                Ok(false)
            }
        }
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
                match conn
                    .and_then(|mut stream| decode(&mut stream))
                    .context("Failed to receive IPC message")
                {
                    Ok(paths) => handle.send(Event::ProcessSingleton { paths }),
                    Err(err) => {
                        handle.send(Event::Error(err));
                        break;
                    }
                }
            }
            log::error!("IPC server has stopped listening due to error");
        });
        log::debug!("Started to listen IPC messages for process singleton");

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
    use std::thread::sleep;
    use std::time::Duration;

    fn send_with_retry(sender: &ProcessSingleton, args: &PathArgs, retry: u8) -> Result<bool> {
        let mut count = 0;
        let mut last_result;
        loop {
            last_result = sender.send(args);
            count += 1;
            if last_result.is_ok() || count > retry {
                return last_result;
            }
            sleep(Duration::from_millis(500));
        }
    }

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

        let expected_args = PathArgs {
            init: Some("foo.md".into()),
            additional_windows: vec!["a.md".into(), "b.md".into()],
            watched: vec!["dir1".into(), "dir2".into()],
        };
        let sender = ProcessSingleton::with_socket_file(&DataDir::new(dir.path()));
        let sent = send_with_retry(&sender, &expected_args, 5).unwrap();
        assert!(sent);

        let request = renderer.recv_timeout(1);
        assert!(
            matches!(
                &request,
                Request::Emit(
                    Event::ProcessSingleton { paths }
                )
                if &expected_args == paths,
            ),
            "request={request:?}, expected_args={expected_args:?}"
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

        let expected_args = PathArgs {
            init: Some("foo.md".into()),
            additional_windows: vec!["a.md".into(), "b.md".into()],
            watched: vec!["dir1".into(), "dir2".into()],
        };
        let sender = ProcessSingleton::with_namespace();
        let sent = send_with_retry(&sender, &expected_args, 5).unwrap();
        assert!(sent);

        let request = renderer.recv_timeout(1);
        assert!(
            matches!(
                &request,
                Request::Emit(
                    Event::ProcessSingleton { paths }
                )
                if &expected_args == paths,
            ),
            "request={request:?}, expected_args={expected_args:?}"
        );
    }

    #[test]
    fn nop_empty_singleton() {
        let args = PathArgs::default();
        let mut singleton = ProcessSingleton::default();
        let sent = singleton.send(&args).unwrap();
        assert!(!sent);

        let renderer = TestRenderer::default();
        singleton.listen(renderer.create_handle()).unwrap();
    }
}
