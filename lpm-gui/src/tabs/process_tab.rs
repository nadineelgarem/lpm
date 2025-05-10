// src/tabs/process_tab.rs
use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Button, CellRendererText, ComboBoxText, Entry, Label, ListStore, Orientation,
    ScrolledWindow, TextView, TreeView, TreeViewColumn,
};
use std::cell::RefCell;
use std::rc::Rc;
use sysinfo::{PidExt, ProcessExt};
use lpm_core::ProcessManager;
use gtk::glib::clone;

pub fn build_process_tab(manager: Rc<RefCell<ProcessManager>>) -> (GtkBox, TextView) {
    let vbox = GtkBox::new(Orientation::Vertical, 5);

    let name_filter = Entry::builder().placeholder_text("Filter by process name...").build();
    let user_filter = Entry::builder().placeholder_text("Filter by user...").build();
    let pid_entry = Entry::builder().placeholder_text("Enter PID...").build();
    let priority_entry = Entry::builder().placeholder_text("Set priority (nice value)...").build();
    let sort_combo = ComboBoxText::new();
    sort_combo.append_text("cpu");
    sort_combo.append_text("memory");
    sort_combo.append_text("pid");
    sort_combo.append_text("name");

    let store = ListStore::new(&[
        u32::static_type(),
        String::static_type(),
        f32::static_type(),
        u64::static_type(),
        String::static_type(),
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

    let kill_button = Button::with_label("Kill Process");
    let restart_button = Button::with_label("Restart Process");
    let alert_button = Button::with_label("Check Alerts");
    let history_button = Button::with_label("Show History");
    let set_priority_button = Button::with_label("Set Priority");
    let count_label = Label::new(Some("Selected: 0 processes"));

    let history_view = TextView::new();
    history_view.set_editable(false);
    history_view.set_cursor_visible(false);
    history_view.set_monospace(true);

    let history_scroll = ScrolledWindow::builder()
        .vexpand(false)
        .hexpand(true)
        .child(&history_view)
        .build();

    let update_display = {
        let manager = Rc::clone(&manager);
        let store = store.clone();
        let name_filter = name_filter.clone();
        let user_filter = user_filter.clone();
        let sort_combo = sort_combo.clone();
        let count_label = count_label.clone();

        move || {
            let mut mgr = manager.borrow_mut();
            let mut processes = if !name_filter.text().is_empty() {
                mgr.list_processes_by_name(&name_filter.text())
            } else if !user_filter.text().is_empty() {
                mgr.list_processes_by_user(&user_filter.text())
            } else {
                mgr.list_processes()
            };

            let sort_key = sort_combo.active_text().unwrap_or_default();
            match sort_key.as_str() {
                "cpu" => processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal)),
                "memory" => processes.sort_by(|a, b| b.memory().cmp(&a.memory())),
                "pid" => processes.sort_by(|a, b| a.pid().cmp(&b.pid())),
                "name" => processes.sort_by(|a, b| a.name().cmp(&b.name())),
                _ => {}
            }

            store.clear();
            for p in &processes {
                let iter = store.append();
                store.set(&iter, &[
                    (0, &p.pid().as_u32()),
                    (1, &p.name().to_string()),
                    (2, &p.cpu_usage()),
                    (3, &p.memory()),
                    (4, &format!("{:?}", p.user_id())),
                ]);
            }

            count_label.set_text(&format!("Shown: {} processes", processes.len()));
        }
    };

    let update_display_rc = Rc::new(update_display);
    name_filter.connect_changed(clone!(@strong update_display_rc => move |_| update_display_rc()));
    user_filter.connect_changed(clone!(@strong update_display_rc => move |_| update_display_rc()));
    sort_combo.connect_changed(clone!(@strong update_display_rc => move |_| update_display_rc()));

    kill_button.connect_clicked(clone!(@strong manager, @strong pid_entry, @strong update_display_rc, @strong history_view => move |_| {
        if let Ok(pid) = pid_entry.text().parse::<usize>() {
            let mut mgr = manager.borrow_mut();
            let msg = if mgr.kill_process(pid) {
                format!("✅ Killed process {}", pid)
            } else {
                format!("⚠️ Failed to kill process {}", pid)
            };
            update_display_rc();
            history_view.buffer().set_text(&msg);
        }
    }));

    restart_button.connect_clicked(clone!(@strong manager, @strong pid_entry, @strong update_display_rc, @strong history_view => move |_| {
        if let Ok(pid) = pid_entry.text().parse::<usize>() {
            let mut mgr = manager.borrow_mut();
            let msg = if mgr.restart_process(pid) {
                format!("✅ Restarted process {}", pid)
            } else {
                format!("⚠️ Failed to restart process {}", pid)
            };
            update_display_rc();
            history_view.buffer().set_text(&msg);
        }
    }));

    set_priority_button.connect_clicked(clone!(@strong manager, @strong pid_entry, @strong priority_entry, @strong history_view => move |_| {
        if let (Ok(pid), Ok(priority)) = (pid_entry.text().parse::<usize>(), priority_entry.text().parse::<i32>()) {
            let mut mgr = manager.borrow_mut();
            let msg = if mgr.change_priority(pid, priority) {
                format!("✅ Changed priority of {} to {}", pid, priority)
            } else {
                format!("⚠️ Failed to change priority for {}", pid)
            };
            history_view.buffer().set_text(&msg);
        }
    }));

    history_button.connect_clicked(clone!(@strong manager, @strong history_view => move |_| {
        let mgr = manager.borrow();
        let text = mgr.show_history();
        history_view.buffer().set_text(&text);
    }));

    let button_box = GtkBox::new(Orientation::Horizontal, 5);
    button_box.append(&kill_button);
    button_box.append(&restart_button);
    button_box.append(&alert_button);
    button_box.append(&history_button);

    vbox.append(&name_filter);
    vbox.append(&user_filter);
    vbox.append(&sort_combo);
    vbox.append(&scrolled_window);
    vbox.append(&button_box);
    vbox.append(&pid_entry);
    vbox.append(&priority_entry);
    vbox.append(&set_priority_button);
    vbox.append(&count_label);
    vbox.append(&history_scroll);

    update_display_rc();

    (vbox, history_view)
}

