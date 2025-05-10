use gtk::prelude::*;
use gtk::{Box as GtkBox, Orientation, ScrolledWindow, TextView, Button};
use std::cell::RefCell;
use std::rc::Rc;
use lpm_core::ProcessManager;
use gtk::glib::clone;

pub fn build_history_tab(manager: Rc<RefCell<ProcessManager>>) -> GtkBox {
    let vbox = GtkBox::new(Orientation::Vertical, 10);

    let history_view = TextView::new();
    history_view.set_editable(false);
    history_view.set_monospace(true);

    let scroll = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&history_view)
        .build();

    // Initial load
    let history_text = manager.borrow().show_history();
    history_view.buffer().set_text(&history_text);

    // Refresh button
    let refresh_button = Button::with_label("Refresh History");
    refresh_button.connect_clicked(clone!(@strong manager, @strong history_view => move |_| {
        let text = manager.borrow().show_history();
        history_view.buffer().set_text(&text);
    }));

    vbox.append(&refresh_button);
    vbox.append(&scroll);
    vbox
}

