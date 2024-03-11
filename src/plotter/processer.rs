use super::histogrammer::histogrammer::{Histogrammer, HistogramTypes};
use super::cutter::cut::CutHandler;
use super::fitter::fit_handler::FitHandler;

use std::collections::HashMap;

use egui_plot::{PlotUi, PlotPoint, Text, Plot, Legend};

#[derive(serde::Deserialize, serde::Serialize)]
struct Processer {
    histogrammer: Histogrammer,
    selected_histograms: Vec<String>,
    cut_handler: CutHandler,

    #[serde(skip)]
    fit_handler: HashMap<String, FitHandler>,
}


impl Processer {
    pub fn new() -> Self {

        Self {
            histogrammer: Histogrammer::new(),
            selected_histograms: Vec::new(),
            cut_handler: CutHandler::new(),
            fit_handler: HashMap::new(),
        }
    }

    pub fn histogrammer_select_histograms(&mut self, ui: &mut egui::Ui) {
        ui.label("Histograms"); // Label for the histogram buttons.
        
        let keys: Vec<String> = self.histogrammer.get_histogram_list(); // Retrieve the list of histogram names.

        // Layout for the buttons: top down and justified at the top.
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::TOP), |ui| {
                for name in keys.iter() {
                    // Determine if this histogram is currently selected.
                    let is_selected = self.selected_histograms.contains(name);

                    // Use selectable_label for selection.
                    // `selectable_label` returns a Response, which we can query for clicks.
                    let response = ui.selectable_label(is_selected, name);

                    // If the label is clicked with the left mouse button, clear the selection and select this histogram.
                    if response.clicked() {
                        self.selected_histograms.clear();
                        self.selected_histograms.push(name.clone());
                    }

                    // If the label is clicked with the right mouse button, toggle this histogram's selection without clearing existing selections.
                    if response.secondary_clicked() {
                        if is_selected {
                            self.selected_histograms.retain(|x| x != name);
                        } else {
                            self.selected_histograms.push(name.clone());
                        }
                    }
                }
            });
        });
    }

    pub fn render_1d_histogram(&mut self, ui: &mut egui::Ui, hist_name: &str) {
        if let Some(HistogramTypes::Hist1D(hist)) = self.histogrammer.histogram_list.get(hist_name) {
            // add a fit handler for this histogram if it doesn't exist
            if !self.fit_handler.contains_key(hist_name) {
                self.fit_handler.insert(hist_name.to_string(), FitHandler::new());
            }

            // Set up the plot for the combined histogram display.
            let plot = Plot::new(hist_name)
                .legend(Legend::default())
                .clamp_grid(true)
                .allow_drag(false)
                .allow_zoom(false)
                .allow_boxed_zoom(true)
                .auto_bounds(egui::Vec2b::new(true, true))
                .allow_scroll(true);

            // check if the ui is in dark mode. 
            // Light blue looks nice on dark mode but hard to see in light mode.
            let color = if ui.ctx().style().visuals.dark_mode {
                egui::Color32::LIGHT_BLUE
            } else {
                egui::Color32::DARK_BLUE
            };

            plot.show(ui, |plot_ui| { 
                let plot_min_x = plot_ui.plot_bounds().min()[0];
                let plot_max_x = plot_ui.plot_bounds().max()[0];
                // let plot_min_y = plot_ui.plot_bounds().min()[1];
                // let plot_max_y = plot_ui.plot_bounds().max()[1];

                if let Some(step_line) = self.histogrammer.egui_histogram_step(hist_name, color) {
                    plot_ui.line(step_line);

                    let stats_entries = hist.legend_entries(plot_min_x, plot_max_x);
                    for (_i, entry) in stats_entries.iter().enumerate() {
                        plot_ui.text(
                            Text::new(PlotPoint::new(0, 0), " ") // Placeholder for positioning; adjust as needed
                                .highlight(false)
                                .color(color)
                                .name(entry)
                        );
                    }
                }

                // add the fit markers and lines to the correct key 
                self.fit_handler.get_mut(hist_name).map(|fit_handler| {
                    fit_handler.markers.cursor_position = plot_ui.pointer_coordinate();
                    fit_handler.markers.draw_markers(plot_ui);
                    fit_handler.draw_fits(plot_ui);
                });

            });
        }
    }

}

// self.fit_handler.markers.cursor_position = plot_ui.pointer_coordinate();
// self.fit_handler.markers.draw_markers(plot_ui);
// self.fit_handler.draw_fits(plot_ui);

/* 
fn histogram_buttons_ui(&mut self, ui: &mut egui::Ui) {
        
    ui.label("Histograms"); // Label for the histogram buttons.
    
    let keys: Vec<String> = self.histogrammer.get_histogram_list(); // Retrieve the list of histogram names.

    // Layout for the buttons: top down and justified at the top.
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::TOP), |ui| {
            for name in keys {
                // Create a button for each histogram name.
                let button: egui::Button<'_> = egui::Button::new(&name);
                let response: egui::Response = ui.add(button); // Add the button to the UI and get the response.

                // If the button is clicked, clear the current selection and select this histogram.
                if response.clicked() {
                    self.selected_histograms.clear();
                    self.selected_histograms.push(name.clone());
                }

                // If the button is right-clicked, add this histogram to the selection without clearing existing selections.
                if response.secondary_clicked() {
                    if !self.selected_histograms.contains(&name) {
                        self.selected_histograms.push(name.clone());
                    }
                }
            }
        });
    });

}
*/