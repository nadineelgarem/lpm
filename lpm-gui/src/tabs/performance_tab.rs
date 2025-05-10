// src/tabs/performance_tab.rs
use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, Orientation, ProgressBar};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use sysinfo::{System, SystemExt};
use gtk::glib;

pub fn build_performance_tab() -> GtkBox {
    let vbox = GtkBox::new(Orientation::Vertical, 10);

    let sys = Arc::new(Mutex::new(System::new_all()));

    let info_label = Label::new(None);
    update_info_label(&info_label, &sys.lock().unwrap());

    let mem_bar = ProgressBar::new();
    mem_bar.set_show_text(true);

    vbox.append(&info_label);
    vbox.append(&mem_bar);

    let sys_clone = Arc::clone(&sys);
    let info_label_clone = info_label.clone();
    let mem_bar_clone = mem_bar.clone();

    gtk::glib::timeout_add_local(Duration::from_secs(1), move || {
        let mut sys = sys_clone.lock().unwrap();
        sys.refresh_memory();

        update_info_label(&info_label_clone, &sys);

        let total = sys.total_memory() as f64;
        let used = sys.used_memory() as f64;
        let percent = used / total;

        mem_bar_clone.set_fraction(percent);
        mem_bar_clone.set_text(Some(&format!("{:.1}%", percent * 100.0)));

        glib::Continue(true)
    });

    vbox
}

fn update_info_label(label: &Label, sys: &System) {
    label.set_text(&format!(
        "Total Memory: {} MB\nUsed Memory: {} MB\nTotal Swap: {} MB\nUsed Swap: {} MB\nUptime: {}s\nCPUs: {}",
        sys.total_memory() / 1024,
        sys.used_memory() / 1024,
        sys.total_swap() / 1024,
        sys.used_swap() / 1024,
        sys.uptime(),
        sys.cpus().len()
    ));
}

