use sysinfo::{ProcessExt, PidExt, System, SystemExt};
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Button, ComboBoxText, Entry, Label,
    Notebook, ScrolledWindow, TextView, TreeView, TreeViewColumn, ListStore, CellRendererText,
};
use glib::clone;
use lpm_core::ProcessManager;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let app = Application::builder()
        .application_id("com.example.lpm_gui")
        .build();

    app.connect_activate(|app| {
        if let Some(settings) = gtk::Settings::default() {
settings.set_gtk_application_prefer_dark_theme(true);
        }

        let window = ApplicationWindow::builder()
            .application(app)
            .title("LPM GUI")
            .default_width(1000)
            .default_height(700)
            .build();

        let notebook = Notebook::new();

        let (process_tab, process_history_view) = build_process_tab();
        let performance_tab = build_performance_tab();
        let history_tab = build_history_tab(process_history_view.clone());

        notebook.append_page(&process_tab, Some(&Label::new(Some("Processes"))));
        notebook.append_page(&performance_tab, Some(&Label::new(Some("Performance"))));
        notebook.append_page(&history_tab, Some(&Label::new(Some("History"))));

        window.set_child(Some(&notebook));
        window.show();
    });

    app.run();
}

fn build_process_tab() -> (GtkBox, TextView) {
    let vbox = GtkBox::new(gtk::Orientation::Vertical, 5);

    let filter_entry = Entry::builder().placeholder_text("Filter by process name...").build();
    let user_entry = Entry::builder().placeholder_text("Filter by user...").build();
    let sort_combo = ComboBoxText::new();
    sort_combo.append_text("cpu");
    sort_combo.append_text("memory");
    sort_combo.append_text("user");

    let store = ListStore::new(&[
        u32::static_type(), String::static_type(), f32::static_type(),
        u64::static_type(), String::static_type(),
    ]);

    let tree_view = TreeView::with_model(&store);
    for (i, title) in ["PID", "Name", "CPU %", "Memory (KB)", "User"].iter().enumerate() {
        let column = TreeViewColumn::new();
        column.set_title(title);
        let cell = CellRendererText::new();
        column.pack_start(&cell, true);
        column.add_attribute(&cell, "text", i as i32);
        tree_view.append_column(&column);
    }

    let scrolled_window = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&tree_view)
        .build();

    let kill_button = Button::with_label(" Kill Process");
    let restart_button = Button::with_label(" Restart Process");
    let export_button = Button::with_label(" Export to JSON");
    let alert_button = Button::with_label("️ Check Alerts");
    let history_button = Button::with_label(" Show History");

    let history_view = TextView::new();
    history_view.set_editable(false);
    history_view.set_cursor_visible(false);
    history_view.set_monospace(true);
    let history_scroll = ScrolledWindow::builder()
        .vexpand(false)
        .hexpand(true)
        .child(&history_view)
        .build();

    let manager = Rc::new(RefCell::new(ProcessManager::new()));

    let update_display = {
        let manager = Rc::clone(&manager);
        let store = store.clone();
        let filter_entry = filter_entry.clone();
        let user_entry = user_entry.clone();
        let sort_combo = sort_combo.clone();

        move || {
            let mut mgr = manager.borrow_mut();
            let mut processes = if !filter_entry.text().is_empty() {
                mgr.list_processes_by_name(&filter_entry.text())
            } else if !user_entry.text().is_empty() {
                mgr.list_processes_by_user(&user_entry.text())
            } else {
                mgr.list_processes()
            };

            let sort_key = sort_combo.active_text().unwrap_or_default();
            if sort_key == "cpu" {
                processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());
            } else if sort_key == "memory" {
                processes.sort_by(|a, b| b.memory().cmp(&a.memory()));
            } else if sort_key == "user" {
                processes.sort_by(|a, b| a.user_id().cmp(&b.user_id()));
            }

            store.clear();
            for p in processes {
                let iter = store.append();
                store.set(
                    &iter,
                    &[
                        (0, &p.pid().as_u32()),
                        (1, &p.name().to_string()),
                        (2, &p.cpu_usage()),
                        (3, &p.memory()),
                        (4, &format!("{:?}", p.user_id())),
                    ],
                );
            }
        }
    };

    let update_display_rc = Rc::new(update_display);

    filter_entry.connect_changed(glib::clone!(@strong update_display_rc => move |_| update_display_rc()));
    user_entry.connect_changed(glib::clone!(@strong update_display_rc => move |_| update_display_rc()));
    sort_combo.connect_changed(glib::clone!(@strong update_display_rc => move |_| update_display_rc()));

    kill_button.connect_clicked(glib::clone!(@strong manager, @strong filter_entry, @strong update_display_rc, @strong history_view => move |_| {
        let pid_str = filter_entry.text().to_string();
        let mut output = String::new();
        if let Ok(pid) = pid_str.parse::<usize>() {
            let mut mgr = manager.borrow_mut();
            if mgr.kill_process(pid) {
                output = format!("✅ Killed process {}", pid);
            } else {
                output = format!("⚠️ Failed to kill process {}", pid);
            }
            update_display_rc();
        } else {
            output = "⚠️ Invalid PID".to_string();
        }
        history_view.buffer().set_text(&output);
    }));

    restart_button.connect_clicked(glib::clone!(@strong manager, @strong filter_entry, @strong update_display_rc, @strong history_view => move |_| {
        let pid_str = filter_entry.text().to_string();
        let mut output = String::new();
        if let Ok(pid) = pid_str.parse::<usize>() {
            let mut mgr = manager.borrow_mut();
            if mgr.restart_process(pid) {
                output = format!("✅ Restarted process {}", pid);
            } else {
                output = format!("⚠️ Failed to restart process {}", pid);
            }
            update_display_rc();
        } else {
            output = "⚠️ Invalid PID".to_string();
        }
        history_view.buffer().set_text(&output);
    }));

    export_button.connect_clicked(glib::clone!(@strong manager, @strong history_view => move |_| {
        let mut mgr = manager.borrow_mut();
        let message = if let Err(e) = mgr.export_processes("json", "processes.json") {
            format!("❌ Export failed: {}", e)
        } else {
            "✅ Exported to processes.json".to_string()
        };
        history_view.buffer().set_text(&message);
    }));

    alert_button.connect_clicked(glib::clone!(@strong manager, @strong history_view => move |_| {
        let mut mgr = manager.borrow_mut();
        let alerts = mgr.check_alerts(80.0, 500_000);
        let message = if alerts.is_empty() {
            "✅ No alerts.".to_string()
        } else {
            format!("⚠️ Alerts:\n{}", alerts.join("\n"))
        };
        history_view.buffer().set_text(&message);
    }));

    history_button.connect_clicked(glib::clone!(@strong manager, @strong history_view => move |_| {
        let mgr = manager.borrow();
        let history = mgr.show_history();
        history_view.buffer().set_text(&history);
    }));

    let button_box = GtkBox::new(gtk::Orientation::Horizontal, 5);
    button_box.append(&kill_button);
    button_box.append(&restart_button);
    button_box.append(&export_button);
    button_box.append(&alert_button);
    button_box.append(&history_button);

    vbox.append(&filter_entry);
    vbox.append(&user_entry);
    vbox.append(&sort_combo);
    vbox.append(&scrolled_window);
    vbox.append(&button_box);
    vbox.append(&history_scroll);

    update_display_rc();

    (vbox, history_view)
}

fn build_performance_tab() -> GtkBox {
    let vbox = GtkBox::new(gtk::Orientation::Vertical, 5);
    let sys = System::new_all();
    let label = Label::new(Some(&format!(
        "Total Memory: {} KB\nUsed Memory: {} KB\nTotal Swap: {} KB\nUsed Swap: {} KB",
        sys.total_memory(),
        sys.used_memory(),
        sys.total_swap(),
        sys.used_swap()
    )));
    vbox.append(&label);
    vbox
}

fn build_history_tab(shared_history_view: TextView) -> GtkBox {
    let vbox = GtkBox::new(gtk::Orientation::Vertical, 5);
    let scrolled_window = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&shared_history_view)
        .build();
    vbox.append(&scrolled_window);
    vbox
}

