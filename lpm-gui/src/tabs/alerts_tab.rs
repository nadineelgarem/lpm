// src/tabs/alerts_tab.rs
use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, Label, ScrolledWindow, TextView, Orientation};
use std::cell::RefCell;
use std::rc::Rc;
use lpm_core::ProcessManager;
use gtk::glib::clone;

pub fn build_alerts_tab(manager: Rc<RefCell<ProcessManager>>) -> GtkBox {
    let vbox = GtkBox::new(Orientation::Vertical, 10);

    let label = Label::new(Some("Check for high CPU or memory usage"));
    let check_button = Button::with_label("Run Alert Check");
    let alert_view = TextView::new();
    alert_view.set_editable(false);
    alert_view.set_monospace(true);

    let scroll = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&alert_view)
        .build();

    check_button.connect_clicked(clone!(@strong manager, @strong alert_view => move |_| {
        let mut mgr = manager.borrow_mut();
        let alerts = mgr.check_alerts(80.0, 500_000); // CPU > 80%, Mem > 500MB (in KB)

        let buffer = alert_view.buffer(); // ✅ no need to unwrap or check Option
        if alerts.is_empty() {
            buffer.set_text("No alerts. All processes are within normal resource usage.");
        } else {
            let output = format!("Alerts detected:\n{}", alerts.join("\n"));
            buffer.set_text(&output);
        }
    }));

    vbox.append(&label);
    vbox.append(&check_button);
    vbox.append(&scroll);
    vbox
}

