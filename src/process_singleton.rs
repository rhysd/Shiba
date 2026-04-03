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
use std::io::{self, ErrorKind, Read, Write};
use std::mem::take;
use std::path::PathBuf;
use std::thread::spawn;

const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024; // 16 MiB

fn encode<W: Write, T: Serialize>(mut writer: W, args: &T) -> io::Result<()> {
    let payload = serde_json::to_vec(args)?;
    if payload.len() > MAX_MESSAGE_SIZE {
        let msg = format!("Could not send too large message ({} bytes > 16 MiB)", payload.len());
        return Err(io::Error::new(ErrorKind::InvalidData, msg));
    }
    let len =
        u32::try_from(payload.len()).expect("IPC message is smaller than 2^32 bytes").to_ne_bytes();
    writer.write_all(&len)?;
    writer.write_all(&payload)?;
    writer.flush()?;
    Ok(())
}

fn decode<R: Read, T: DeserializeOwned>(mut reader: R) -> io::Result<T> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    let len = u32::from_ne_bytes(buf) as usize;
    if len > MAX_MESSAGE_SIZE {
        let msg = format!("Could not receive too large message ({len} bytes > 16 MiB)");
        return Err(io::Error::new(ErrorKind::InvalidInput, msg));
    }
    let mut buf = vec![0; len];
    reader.read_exact(&mut buf)?;
    let msg = serde_json::from_slice(&buf)?;
    Ok(msg)
}

#[derive(Default)]
pub struct ProcessSingleton {
    name: Option<Name<'static>>,
    path: Option<PathBuf>,
    cleanup: bool,
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
            Ok(name) => Self { name: Some(name), path: Some(path), cleanup: false },
            Err(err) => {
                log::error!("Could not create a socket file {path:?} for IPC: {err:?}");
                Self::default()
            }
        }
    }

    #[cfg(target_os = "windows")]
    pub fn with_default_namespace() -> Self {
        Self::with_namespace("shiba.singleton.sock")
    }

    #[cfg(target_os = "windows")]
    pub fn with_namespace(name: &'static str) -> Self {
        match name.to_ns_name::<GenericNamespaced>() {
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

    fn remove_socket_file(&self) {
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
        self.remove_socket_file();

        let listener = ListenerOptions::new()
            .name(name)
            .create_sync()
            .context("Could not listen IPC messages")?;

        self.cleanup = true;

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
        if self.cleanup {
            self.remove_socket_file();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{Event, Renderer, Request};
    use crate::test::TestRenderer;
    use std::iter::repeat_n;
    use std::thread::sleep;
    use std::time::Duration;

    fn send_with_retry(sender: &ProcessSingleton, args: &PathArgs, retry: u8) -> Result<bool> {
        let mut count = 0;
        let mut last_result;
        loop {
            last_result = sender.send(args);
            count += 1;
            if last_result.as_ref().is_ok_and(|&sent| sent) || count > retry {
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
            let singleton = ProcessSingleton::with_default_namespace();
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

        let request = renderer.recv_timeout(Duration::from_secs(1));
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
        let mut listener = ProcessSingleton::with_default_namespace();
        assert!(listener.can_listen());

        listener.listen(renderer.create_handle()).unwrap();
        assert!(!listener.can_listen());

        let expected_args = PathArgs {
            init: Some("foo.md".into()),
            additional_windows: vec!["a.md".into(), "b.md".into()],
            watched: vec!["dir1".into(), "dir2".into()],
        };
        let sender = ProcessSingleton::with_default_namespace();
        let sent = send_with_retry(&sender, &expected_args, 5).unwrap();
        assert!(sent);

        let request = renderer.recv_timeout(Duration::from_secs(1));
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

    #[test]
    fn send_too_large_message() {
        #[cfg(target_os = "windows")]
        let sender = ProcessSingleton::with_namespace("shiba.test.error.send");
        #[cfg(not(target_os = "windows"))]
        let (sender, _tempdir) = {
            let dir = tempfile::tempdir().unwrap();
            let sender = ProcessSingleton::with_socket_file(&DataDir::new(dir.path()));
            (sender, dir)
        };

        let renderer = TestRenderer::default();
        let mut listener =
            ProcessSingleton { name: sender.name.clone(), path: None, cleanup: false };
        listener.listen(renderer.create_handle()).unwrap();

        let path = String::from_utf8(vec![b'a'; 16 * 1024]).unwrap().into();
        let too_large_args =
            PathArgs { watched: repeat_n(path, 1024).collect(), ..Default::default() };

        let mut last_error = String::new();
        for _ in 0..10 {
            sleep(Duration::from_millis(200));
            let msg = match sender.send(&too_large_args) {
                Ok(false) => continue,
                Ok(true) => panic!("Socket was unexpectedly connected"),
                Err(err) => format!("{err}"),
            };
            if msg.contains("Could not send too large message") {
                return; // Test passed
            }
            last_error = msg;
        }
        panic!("Expected error did not occur after 2 seconds. last error: {last_error:?}");
    }

    #[test]
    fn receive_too_large_message() {
        #[cfg(target_os = "windows")]
        let mut listener = ProcessSingleton::with_namespace("shiba.test.error.receive");
        #[cfg(not(target_os = "windows"))]
        let (mut listener, _tempdir) = {
            let dir = tempfile::tempdir().unwrap();
            let listener = ProcessSingleton::with_socket_file(&DataDir::new(dir.path()));
            (listener, dir)
        };

        let name = listener.name.clone().unwrap();
        let renderer = TestRenderer::default();
        listener.listen(renderer.create_handle()).unwrap();

        let mut count = 0;
        let mut conn = loop {
            count += 1;
            match Stream::connect(name.clone()) {
                Ok(conn) => break conn,
                Err(err) if count >= 10 => {
                    panic!("Could not connect to server after 2 seconds: {err}")
                }
                Err(_) => {
                    sleep(Duration::from_millis(200));
                }
            }
        };

        let too_large_len = (16u32 * 1024 * 1024 + 1).to_ne_bytes();
        conn.write_all(&too_large_len).unwrap();

        let request = renderer.recv_timeout(Duration::from_secs(1));
        assert!(
            matches!(
                &request,
                Request::Emit(Event::Error(err))
                if err
                    .chain()
                    .any(|err| format!("{err}").contains("Could not receive too large message")),
            ),
            "unexpected request: {request:?}",
        );
    }
}
