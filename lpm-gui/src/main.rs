use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Label, Notebook};
use std::cell::RefCell;
use std::rc::Rc;
use lpm_core::ProcessManager;

mod tabs {
    pub mod process_tab;
    pub mod performance_tab;
    pub mod process_tree_tab;
    pub mod alerts_tab;
    pub mod history_tab;
    pub mod graph_tab; // ✅ Added
}

use tabs::process_tab::build_process_tab;
use tabs::performance_tab::build_performance_tab;
use tabs::process_tree_tab::build_process_tree_tab;
use tabs::alerts_tab::build_alerts_tab;
use tabs::history_tab::build_history_tab;
use tabs::graph_tab::build_graph_tab; // ✅ Added

fn main() {
    let app = Application::builder()
        .application_id("com.example.lpm_gui")
        .build();

    app.connect_activate(|app| {
        // Optional: Enable dark theme
        if let Some(settings) = gtk::Settings::default() {
            settings.set_gtk_application_prefer_dark_theme(true);
        }

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Linux Process Manager (LPM)")
            .default_width(1100)
            .default_height(750)
            .build();

        let notebook = Notebook::new();
        let manager = Rc::new(RefCell::new(ProcessManager::new()));

        // Build and add each tab
        let (process_tab, _history_view) = build_process_tab(Rc::clone(&manager));
        let performance_tab = build_performance_tab();
        let tree_tab = build_process_tree_tab(Rc::clone(&manager));
        let alerts_tab = build_alerts_tab(Rc::clone(&manager));
        let history_tab = build_history_tab(Rc::clone(&manager));
        let graph_tab = build_graph_tab(); // ✅ Graphs Tab

        notebook.append_page(&process_tab, Some(&Label::new(Some("Processes"))));
        notebook.append_page(&performance_tab, Some(&Label::new(Some("Performance"))));
        notebook.append_page(&tree_tab, Some(&Label::new(Some("Tree"))));
        notebook.append_page(&alerts_tab, Some(&Label::new(Some("Alerts"))));
        notebook.append_page(&history_tab, Some(&Label::new(Some("History"))));
        notebook.append_page(&graph_tab, Some(&Label::new(Some("Graphs")))); // ✅ Graphs tab visible

        window.set_child(Some(&notebook));
        window.show();
    });

    app.run();
}

