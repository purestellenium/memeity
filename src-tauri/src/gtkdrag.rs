//! Dragging a meme out of the pane on Linux (GTK 3).
//!
//! The `drag` crate's GTK backend never delivers the file: it starts the drag with the
//! button mask where GTK wants the button number (crabnebula-dev/drag-rs#103), and it
//! disconnects its data handler as soon as the drop is sent, before the target has asked
//! for the data (drag-rs#84). This is the same drag, cleaned up only once GTK says it's over.

use gtk::{gdk, gdk_pixbuf, glib, prelude::*};
use std::cell::{Cell, OnceCell, RefCell};
use std::path::Path;
use std::rc::Rc;

/// Starts dragging `file`, with `icon` (PNG bytes) under the cursor. `on_end` runs once
/// the drag is over, with whether it was dropped on something that took it.
pub fn start(
    window: &gtk::ApplicationWindow,
    file: &Path,
    icon: &[u8],
    on_end: impl Fn(bool) + 'static,
) -> Result<(), String> {
    let uri = glib::filename_to_uri(file, None).map_err(|e| e.to_string())?;
    let targets = gtk::TargetList::new(&[]);
    targets.add_uri_targets(0);

    let handlers: Rc<RefCell<Vec<glib::SignalHandlerId>>> = Rc::default();
    // drag-failed comes before drag-end for anything but a drop the target took: no target,
    // Esc, a timeout. (`drag_drop_succeeded()` can't say: it's always false on Wayland, and
    // true after a cancel on X11.)
    let failed = Rc::new(Cell::new(false));
    // These handlers are on the window, which may be running another drag too (one started
    // while this one snaps back): each only answers for its own drag.
    let ours_cell: Rc<OnceCell<gdk::DragContext>> = Rc::default();
    let is_ours = {
        let ours = ours_cell.clone();
        move |context: &gdk::DragContext| ours.get() == Some(context)
    };

    // The target asks for the data after the drop, so this stays connected until drag-end.
    let data_get = {
        let is_ours = is_ours.clone();
        window.connect_drag_data_get(move |_, context, data, _, _| {
            if is_ours(context) {
                data.set_uris(&[uri.as_str()]);
            }
        })
    };
    let fail = {
        let (failed, is_ours) = (failed.clone(), is_ours.clone());
        window.connect_drag_failed(move |_, context, _| {
            if is_ours(context) {
                failed.set(true);
            }
            glib::Propagation::Proceed // the usual snap-back animation
        })
    };
    let end = {
        let handlers = handlers.clone();
        window.connect_drag_end(move |window, context| {
            if !is_ours(context) {
                return;
            }
            for id in handlers.take() {
                window.disconnect(id);
            }
            on_end(!failed.get());
        })
    };
    handlers.borrow_mut().extend([data_get, fail, end]);

    // Button 1: the pane only starts drags from a primary-button press.
    let Some(context) = window.drag_begin_with_coordinates(&targets, gdk::DragAction::COPY, 1, None, -1, -1) else {
        for id in handlers.take() {
            window.disconnect(id);
        }
        return Err("GTK didn't start the drag.".into());
    };
    let _ = ours_cell.set(context.clone());
    if let Some(pixbuf) = pixbuf(icon) {
        context.drag_set_icon_pixbuf(&pixbuf, pixbuf.width() / 2, pixbuf.height() / 2);
    }
    Ok(())
}

fn pixbuf(png: &[u8]) -> Option<gdk_pixbuf::Pixbuf> {
    let loader = gdk_pixbuf::PixbufLoader::new();
    loader.write(png).ok()?;
    loader.close().ok()?;
    loader.pixbuf()
}
