// src/tabs/process_tree_tab.rs

use gtk::prelude::*;
use gtk::{
    Box as GtkBox, CellRendererText, Orientation, ScrolledWindow,
    TreeStore, TreeView, TreeViewColumn,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use lpm_core::ProcessManager;

pub fn build_process_tree_tab(manager: Rc<RefCell<ProcessManager>>) -> GtkBox {
    let vbox = GtkBox::new(Orientation::Vertical, 5);

    // TreeStore columns: PID (u32), Name (String), Parent PID (u32)
    let tree_store = TreeStore::new(&[
        u32::static_type(),       // PID
        String::static_type(),    // Name
        u32::static_type(),       // Parent PID
    ]);

    let tree_view = TreeView::with_model(&tree_store);

    let column_titles = ["PID", "Name", "Parent PID"];
    for (i, title) in column_titles.iter().enumerate() {
        let column = TreeViewColumn::new();
        column.set_title(title);
        let cell = CellRendererText::new();
        column.pack_start(&cell, true);
        column.add_attribute(&cell, "text", i as i32);
        tree_view.append_column(&column);
    }

    let scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&tree_view)
        .build();

    // Get the tree structure data
    let mut manager_ref = manager.borrow_mut();
    let process_data = manager_ref.get_process_tree();

    let mut pid_to_iter = HashMap::new();

    for (pid_usize, name, ppid_opt_usize) in process_data {
        let pid = pid_usize as u32;
        let ppid = ppid_opt_usize.unwrap_or(0) as u32;

        let iter = if let Some(parent_iter) = pid_to_iter.get(&ppid) {
            tree_store.append(Some(parent_iter))
        } else {
            tree_store.append(None)
        };

        tree_store.set(&iter, &[
            (0, &pid),
            (1, &name),
            (2, &ppid),
        ]);

        pid_to_iter.insert(pid, iter);
    }

    vbox.append(&scrolled);
    vbox
}

