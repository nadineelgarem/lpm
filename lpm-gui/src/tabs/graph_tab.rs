// src/tabs/graph_tab.rs
use gtk::prelude::*;
use gtk::{Box as GtkBox, DrawingArea as GtkDrawingArea, Orientation};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use sysinfo::{CpuExt, System, SystemExt};
use gtk::glib;
use plotters::prelude::*;
use plotters_cairo::CairoBackend;
use plotters::coord::Shift;
use cairo::Context as CairoContext; // ✅ Correct cairo context

pub fn build_graph_tab() -> GtkBox {
    let vbox = GtkBox::new(Orientation::Vertical, 10);

    let line_row = GtkBox::new(Orientation::Horizontal, 10);
    let pie_row = GtkBox::new(Orientation::Horizontal, 10);

    let cpu_chart = GtkDrawingArea::new();
    cpu_chart.set_content_width(600);
    cpu_chart.set_content_height(300);

    let memory_chart = GtkDrawingArea::new();
    memory_chart.set_content_width(600);
    memory_chart.set_content_height(300);

    let pie_chart = GtkDrawingArea::new();
    pie_chart.set_content_width(300);
    pie_chart.set_content_height(300);

    let swap_pie_chart = GtkDrawingArea::new();
    swap_pie_chart.set_content_width(300);
    swap_pie_chart.set_content_height(300);

    line_row.append(&cpu_chart);
    line_row.append(&memory_chart);

    pie_row.append(&pie_chart);
    pie_row.append(&swap_pie_chart);

    let cpu_data = Rc::new(RefCell::new(vec![0f32; 60]));
    let mem_data = Rc::new(RefCell::new(vec![0f32; 60]));
    let system = Rc::new(RefCell::new(System::new_all()));

    let cpu_data_clone = cpu_data.clone();
    let mem_data_clone = mem_data.clone();
    let system_clone = system.clone();
    let cpu_chart_clone = cpu_chart.clone();
    let memory_chart_clone = memory_chart.clone();
    let pie_chart_clone = pie_chart.clone();
    let swap_pie_clone = swap_pie_chart.clone();

    glib::timeout_add_local(Duration::from_secs(1), move || {
        let mut sys = system_clone.borrow_mut();
        sys.refresh_cpu();
        sys.refresh_memory();

        let cpu = sys.global_cpu_info().cpu_usage();
        let used_mem = sys.used_memory() as f32;
        let total_mem = sys.total_memory() as f32;
        let mem_percent = (used_mem / total_mem) * 100.0;

        let used_swap = sys.used_swap() as f32;
        let total_swap = sys.total_swap() as f32;

        {
            let mut c = cpu_data_clone.borrow_mut();
            c.remove(0);
            c.push(cpu);
        }
        {
            let mut m = mem_data_clone.borrow_mut();
            m.remove(0);
            m.push(mem_percent);
        }

        draw_line_chart(&cpu_chart_clone, cpu_data_clone.borrow().clone(), "CPU Usage (%) over Time");
        draw_line_chart(&memory_chart_clone, mem_data_clone.borrow().clone(), "Memory Usage (%) over Time");
        draw_pie_chart(&pie_chart_clone, used_mem, total_mem, String::from("Memory"));
        draw_pie_chart(&swap_pie_clone, used_swap, total_swap, String::from("Swap"));

        glib::Continue(true)
    });

    vbox.append(&line_row);
    vbox.append(&pie_row);

    vbox
}

fn draw_line_chart(area: &GtkDrawingArea, data: Vec<f32>, label: &str) {
    let label = label.to_string();
    area.set_draw_func(move |_, cr, width, height| {
        let cairo_ctx = CairoContext::new(cr.target()).unwrap();
        let backend = CairoBackend::new(&cairo_ctx, (width as u32, height as u32)).unwrap();
        let root = backend.into_drawing_area();
        root.fill(&WHITE).unwrap();

        let _ = ChartBuilder::on(&root)
            .margin(10)
            .caption(&label, ("sans-serif", 14))
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(0i32..60i32, 0f32..100.0)
            .and_then(|mut chart| {
                chart.configure_mesh()
                    .x_desc("Seconds")
                    .y_desc(label.clone())
                    .disable_mesh()
                    .draw()?;
                chart.draw_series(LineSeries::new(
                    data.iter().enumerate().map(|(i, v)| (i as i32, *v)),
                    &RED,
                ))?;
                Ok(())
            });

        root.present().unwrap();
    });
}

fn draw_pie_chart(area: &GtkDrawingArea, used: f32, total: f32, label: String) {
    area.set_draw_func(move |_, cr, width, height| {
        let cairo_ctx = CairoContext::new(cr.target()).unwrap();
        let backend = CairoBackend::new(&cairo_ctx, (width as u32, height as u32)).unwrap();
        let root = backend.into_drawing_area();
        root.fill(&WHITE).unwrap();

        let percent_used = used / total;
        let percent_free = 1.0 - percent_used;

        let center = ((width / 2) as i32, (height / 2) as i32);
        let radius = width.min(height) as i32 / 3;

        let mut start_angle = 0.0_f64;
        let used_angle = (360.0 * percent_used) as f64;
        let free_angle = (360.0 * percent_free) as f64;

        draw_sector(&root, center, radius, start_angle, used_angle, &RED);
        start_angle += used_angle;
        draw_sector(&root, center, radius, start_angle, free_angle, &GREEN);

        use plotters::style::TextStyle;
        root.draw_text(
            &format!("Used {}: {:.1}%", label, percent_used * 100.0),
            &TextStyle::from(("sans-serif", 14).into_font()).color(&BLACK),
            (center.0 - 60, center.1 + radius + 20),
        ).ok();

        root.draw_text(
            &format!("Free {}: {:.1}%", label, percent_free * 100.0),
            &TextStyle::from(("sans-serif", 14).into_font()).color(&BLACK),
            (center.0 - 60, center.1 + radius + 40),
        ).ok();

        root.present().unwrap();
    });
}

fn draw_sector<DB: DrawingBackend>(
    root: &DrawingArea<DB, Shift>,
    center: (i32, i32),
    radius: i32,
    start: f64,
    end: f64,
    color: &RGBColor,
) {
    use plotters::element::PathElement;

    let steps = 100;
    let delta = end / steps as f64;
    for i in 0..steps {
        let a0 = start + i as f64 * delta;
        let a1 = a0 + delta;

        let x0 = center.0 + (radius as f64 * a0.to_radians().cos()) as i32;
        let y0 = center.1 + (radius as f64 * a0.to_radians().sin()) as i32;
        let x1 = center.0 + (radius as f64 * a1.to_radians().cos()) as i32;
        let y1 = center.1 + (radius as f64 * a1.to_radians().sin()) as i32;

        root.draw(&PathElement::new(vec![center, (x0, y0), (x1, y1), center], color.filled())).unwrap();
    }
}

