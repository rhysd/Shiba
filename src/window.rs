use crate::preview::Preview;
use crate::renderer::Renderer;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<Id> {
    UnknownWindow(Id),
    NoWindowAvailable,
}

impl<Id: fmt::Debug> fmt::Display for Error<Id> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownWindow(id) => write!(f, "unknown window ID: {id:?}"),
            Self::NoWindowAvailable => write!(f, "no window is available"),
        }
    }
}

impl<Id: fmt::Debug> std::error::Error for Error<Id> {}

pub struct WindowManager<R: Renderer> {
    windows: HashMap<R::WindowId, (R::Window, Preview)>,
    focused: Option<R::WindowId>,
}

impl<R: Renderer> Default for WindowManager<R> {
    fn default() -> Self {
        Self { windows: HashMap::new(), focused: None }
    }
}

impl<R: Renderer> WindowManager<R> {
    pub fn focused(&self) -> Result<(&R::Window, &Preview), Error<R::WindowId>> {
        if let Some(id) = self.focused {
            self.get(id)
        } else if let Some((win, prev)) = self.windows.values().next() {
            Ok((win, prev)) // Fallback
        } else {
            Err(Error::NoWindowAvailable)
        }
    }

    pub fn get(&self, id: R::WindowId) -> Result<(&R::Window, &Preview), Error<R::WindowId>> {
        self.windows.get(&id).map(|(win, prev)| (win, prev)).ok_or(Error::UnknownWindow(id))
    }

    pub fn get_mut(
        &mut self,
        id: R::WindowId,
    ) -> Result<(&mut R::Window, &mut Preview), Error<R::WindowId>> {
        self.windows.get_mut(&id).map(|(win, prev)| (win, prev)).ok_or(Error::UnknownWindow(id))
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (R::WindowId, &mut R::Window, &mut Preview)> {
        self.windows.iter_mut().map(|(&i, (w, p))| (i, w, p))
    }

    pub fn is_last(&self, id: R::WindowId) -> bool {
        self.windows.len() == 1 && self.windows.contains_key(&id)
    }

    pub fn add(&mut self, id: R::WindowId, window: R::Window) {
        log::debug!("Add new window: {id:?}");
        self.windows.insert(id, (window, Preview::default()));
        self.set_focus(id);
    }

    pub fn remove(&mut self, id: R::WindowId) -> Option<(R::Window, Preview)> {
        let removed = self.windows.remove(&id)?;
        log::debug!("Removed window {id:?}");
        if self.focused == Some(id) {
            log::debug!("Focus was lost because the window was removed");
            self.focused = None;
        }
        Some(removed)
    }

    pub fn remove_others(
        &mut self,
        id: R::WindowId,
    ) -> impl Iterator<Item = (R::WindowId, R::Window, Preview)> {
        self.focused = Some(id);
        self.windows.extract_if(move |i, _| *i != id).map(|(i, (w, p))| (i, w, p))
    }

    pub fn set_focus(&mut self, id: R::WindowId) {
        if self.focused == Some(id) {
            return;
        }
        if !self.windows.contains_key(&id) {
            log::error!("Unknown window is focused: {id:?}");
            return;
        }
        log::debug!("Update focus: {:?} -> {:?}", self.focused, id);
        self.focused = Some(id);
    }

    pub fn focused_id(&self) -> Result<R::WindowId, Error<R::WindowId>> {
        if let Some(id) = self.focused {
            Ok(id)
        } else {
            self.windows.keys().next().copied().ok_or(Error::NoWindowAvailable)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{TestRenderer, TestWindow};

    fn add(wm: &mut WindowManager<TestRenderer>) -> u32 {
        let w = TestWindow::new();
        let id = w.window_id;
        wm.add(id, w);
        id
    }

    #[test]
    fn add_and_remove() {
        let wm = &mut WindowManager::<TestRenderer>::default();

        let id = add(wm);
        assert_eq!(wm.get(id).unwrap().0.window_id, id);
        assert_eq!(wm.get_mut(id).unwrap().0.window_id, id);

        let id2 = add(wm);
        assert_eq!(wm.remove(id2).unwrap().0.window_id, id2); // Focus is lost here
        assert!(wm.remove(id2).is_none()); // Focus is lost here

        assert!(matches!(wm.get(id2), Err(Error::UnknownWindow(i)) if i == id2));
        assert!(matches!(wm.get_mut(id2), Err(Error::UnknownWindow(i)) if i == id2));

        // Removing an invalid ID does nothing
        assert!(wm.remove(id + 100).is_none());

        assert_eq!(wm.remove(id).unwrap().0.window_id, id);
        assert!(wm.remove(id).is_none()); // Focus is lost here
    }

    #[test]
    fn manage_focus() {
        let wm = &mut WindowManager::<TestRenderer>::default();
        let id = add(wm);

        // Fallback to the first window when focus is not set
        assert_eq!(wm.focused_id().unwrap(), id);
        assert_eq!(wm.focused().unwrap().0.window_id, id);

        wm.set_focus(id);
        assert_eq!(wm.focused_id().unwrap(), id);

        wm.set_focus(id); // This does nothing
        assert_eq!(wm.focused_id().unwrap(), id);

        wm.set_focus(id + 1); // Do nothing on invalid window ID
        assert_eq!(wm.focused_id().unwrap(), id);

        // Add the second window
        let id2 = add(wm);

        assert_eq!(wm.focused_id().unwrap(), id2);
        assert_eq!(wm.focused().unwrap().0.window_id, id2);

        wm.set_focus(id); // Set focus back to the first window
        assert_eq!(wm.focused_id().unwrap(), id);

        wm.remove(id); // Remove the first window

        // Fallback to the second window
        assert_eq!(wm.focused_id().unwrap(), id2);
        assert_eq!(wm.focused().unwrap().0.window_id, id2);

        wm.set_focus(id2);
        assert_eq!(wm.focused_id().unwrap(), id2);
        assert_eq!(wm.focused().unwrap().0.window_id, id2);

        // This no longer does nothing
        wm.set_focus(id);
        assert_eq!(wm.focused_id().unwrap(), id2);
    }

    #[test]
    fn remove_unfocused_window() {
        let wm = &mut WindowManager::<TestRenderer>::default();
        let (id1, id2, id3) = (add(wm), add(wm), add(wm));
        assert_eq!(wm.focused_id().unwrap(), id3);
        wm.remove(id2);
        assert_eq!(wm.focused_id().unwrap(), id3);
        wm.remove(id1);
        assert_eq!(wm.focused_id().unwrap(), id3);
    }

    #[test]
    fn iter_windows() {
        let wm = &mut WindowManager::<TestRenderer>::default();
        let expected = [add(wm), add(wm), add(wm)];
        let mut actual = wm
            .iter_mut()
            .map(|(id, w, _)| {
                assert_eq!(id, w.window_id);
                id
            })
            .collect::<Vec<_>>();
        actual.sort(); // Iteration order is random
        assert_eq!(&expected, actual.as_slice());
    }

    #[test]
    fn remove_other_windows() {
        let wm = &mut WindowManager::<TestRenderer>::default();
        let (id1, id2, id3) = (add(wm), add(wm), add(wm));
        wm.set_focus(id2);
        let mut removed: Vec<_> = wm.remove_others(id2).map(|(id, _, _)| id).collect();
        removed.sort();
        assert_eq!(removed, &[id1, id3]);
        assert_eq!(wm.focused_id().unwrap(), id2);
    }

    #[test]
    fn check_last_window() {
        let invalid_id = 100;
        let wm = &mut WindowManager::<TestRenderer>::default();
        assert!(!wm.is_last(0));

        let id1 = add(wm);
        assert!(wm.is_last(id1));
        assert!(!wm.is_last(invalid_id));

        let id2 = add(wm);
        assert!(!wm.is_last(id1));
        assert!(!wm.is_last(id2));
        assert!(!wm.is_last(invalid_id));

        wm.remove(id1);
        assert!(!wm.is_last(id1));
        assert!(wm.is_last(id2));
        assert!(!wm.is_last(invalid_id));
    }
}
