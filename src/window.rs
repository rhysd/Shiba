use crate::preview::Preview;
use crate::renderer::Renderer;
use std::collections::HashMap;

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
    pub fn focused(&self) -> (&R::Window, &Preview) {
        let (win, prev) = if let Some(id) = &self.focused {
            self.windows.get(id).expect("focused window must exist")
        } else {
            self.windows.values().next().expect("at least one window exist")
        };
        (win, prev)
    }

    pub fn focused_mut(&mut self) -> (&mut R::Window, &mut Preview) {
        let (win, prev) = if let Some(id) = &self.focused {
            self.windows.get_mut(id).expect("focused window must exist")
        } else {
            self.windows.values_mut().next().expect("at least one window exist")
        };
        (win, prev)
    }

    pub fn get(&self, id: R::WindowId) -> (&R::Window, &Preview) {
        if let Some((win, prev)) = self.windows.get(&id) { (win, prev) } else { self.focused() }
    }

    pub fn get_mut(&mut self, id: R::WindowId) -> (&mut R::Window, &mut Preview) {
        if let Some(ptr) = self.windows.get_mut(&id).map(|v| v as *mut _) {
            // Safety:
            // `ptr` originates from `self.windows.get_mut(&key)`, so it points to a valid element
            // inside `self.windows`. Converting it to a raw pointer ends the borrow from `get_mut`.
            // In this branch we immediately reconstruct `&mut V` and return it without mutating the
            // map or creating any other mutable references to the same element. The `else` branch
            // (which calls `focused_mut`) is not executed in this case, so no aliasing mutable
            // reference can occur.
            let (win, prev) = unsafe { &mut *ptr };
            (win, prev)
        } else {
            self.focused_mut()
        }
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (R::WindowId, &mut R::Window, &mut Preview)> {
        self.windows.iter_mut().map(|(&i, (w, p))| (i, w, p))
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
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

    pub fn focused_id(&self) -> R::WindowId {
        self.focused
            .unwrap_or_else(|| *self.windows.keys().next().expect("at least one window exists"))
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
        assert!(wm.is_empty());

        let id = add(wm);
        assert!(!wm.is_empty());
        assert_eq!(wm.get(id).0.window_id, id);
        assert_eq!(wm.get_mut(id).0.window_id, id);

        let id2 = add(wm);
        assert_eq!(wm.remove(id2).unwrap().0.window_id, id2); // Focus is lost here
        assert!(!wm.is_empty());

        // Fallback
        assert_eq!(wm.get(id2).0.window_id, id);
        assert_eq!(wm.get_mut(id2).0.window_id, id);

        // Removing an invalid ID does nothing
        assert!(wm.remove(id + 100).is_none());

        assert_eq!(wm.remove(id).unwrap().0.window_id, id);
        assert!(wm.is_empty());
    }

    #[test]
    fn manage_focus() {
        let wm = &mut WindowManager::<TestRenderer>::default();
        let id = add(wm);

        // Fallback to the first window when focus is not set
        assert_eq!(wm.focused_id(), id);
        assert_eq!(wm.focused().0.window_id, id);
        assert_eq!(wm.focused_mut().0.window_id, id);

        wm.set_focus(id);
        assert_eq!(wm.focused_id(), id);

        wm.set_focus(id); // This does nothing
        assert_eq!(wm.focused_id(), id);

        wm.set_focus(id + 1); // Do nothing on invalid window ID
        assert_eq!(wm.focused_id(), id);

        // Add the second window
        let id2 = add(wm);

        assert_eq!(wm.focused_id(), id2);
        assert_eq!(wm.focused().0.window_id, id2);
        assert_eq!(wm.focused_mut().0.window_id, id2);

        wm.set_focus(id); // Set focus back to the first window
        assert_eq!(wm.focused_id(), id);

        wm.remove(id); // Remove the first window

        // Fallback to the second window
        assert_eq!(wm.focused_id(), id2);
        assert_eq!(wm.focused().0.window_id, id2);
        assert_eq!(wm.focused_mut().0.window_id, id2);

        wm.set_focus(id2);
        assert_eq!(wm.focused_id(), id2);
        assert_eq!(wm.focused().0.window_id, id2);
        assert_eq!(wm.focused_mut().0.window_id, id2);

        // This no longer does nothing
        wm.set_focus(id);
        assert_eq!(wm.focused_id(), id2);
    }

    #[test]
    fn remove_unfocused_window() {
        let wm = &mut WindowManager::<TestRenderer>::default();
        let (id1, id2, id3) = (add(wm), add(wm), add(wm));
        assert_eq!(wm.focused_id(), id3);
        wm.remove(id2);
        assert_eq!(wm.focused_id(), id3);
        wm.remove(id1);
        assert_eq!(wm.focused_id(), id3);
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
        assert_eq!(wm.focused_id(), id2);
        assert!(!wm.is_empty());
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
