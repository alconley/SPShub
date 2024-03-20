use super::histogrammer::histogrammer::{Histogrammer, HistogramTypes};
use super::histogrammer::histogram_script::add_histograms;
use super::cutter::cut_handler::CutHandler;
use super::fitter::fit_handler::FitHandler;
use super::lazyframer::LazyFramer;

use std::collections::HashMap;
use std::path::PathBuf;

use egui_plot::{PlotPoint, Text, Plot, Legend};

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Processer {
    #[serde(skip)]
    pub lazyframer: Option<LazyFramer>,

    pub files: Vec<PathBuf>,
    pub histogrammer: Histogrammer,
    pub selected_histograms: Vec<String>,
    pub cut_handler: CutHandler,
    pub selected_histogram: String,
    pub fit_handler: HashMap<String, FitHandler>,
}


impl Processer {
    pub fn new() -> Self {

        Self {
            lazyframer: None,
            files: Vec::new(),
            histogrammer: Histogrammer::new(),
            selected_histograms: Vec::new(),
            cut_handler: CutHandler::new(),
            selected_histogram: String::new(),
            fit_handler: HashMap::new(),
        }
    }

    fn create_lazyframe(&mut self) {
        self.lazyframer = Some(LazyFramer::new(self.files.clone()));

        // Update CutHandler with column names from LazyFramer
        if let Some(ref lazyframer) = self.lazyframer {
            let column_names = lazyframer.get_column_names();
            self.cut_handler.update_column_names(column_names);
            log::info!("Column names: {:?}", self.cut_handler.column_names.clone());
        }
    }

    fn perform_histogrammer_from_lazyframe(&mut self) {
        if let Some(lazyframer) = &self.lazyframer {
            if let Some(lf) = &lazyframer.lazyframe {
                match add_histograms(lf.clone()) { 
                    Ok(h) => {
                        self.histogrammer = h;
                    },
                    Err(e) => {
                        log::error!("Failed to create histograms: {}", e);
                    }
                }
            } else {
                log::error!("LazyFrame is not loaded");
            }
        } else {
            log::error!("LazyFramer is not initialized");
        }
    }

    pub fn select_histograms_ui(&mut self, ui: &mut egui::Ui) {
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

        if let Some(selected_name) = self.selected_histograms.first() {
            self.selected_histogram = selected_name.clone();
        }
    }

    pub fn render_1d_histogram(&mut self, ui: &mut egui::Ui) {
        if let Some(hist_name) = self.selected_histograms.first() {
            if let Some(HistogramTypes::Hist1D(hist)) = self.histogrammer.histogram_list.get(hist_name.as_str()) {
                
                let fit_handler = self.fit_handler.entry(hist_name.clone()).or_insert_with(FitHandler::new);
                fit_handler.histogram = Some(hist.clone()); // Set the histogram for the fit handler
                fit_handler.interactive_keybinds(ui); // enable the key binds to add markers and draw the fits

                let plot = Plot::new(&hist_name)
                    .legend(Legend::default())
                    .clamp_grid(true)
                    .allow_drag(false)
                    .allow_zoom(false)
                    .allow_boxed_zoom(true)
                    .auto_bounds(egui::Vec2b::new(true, true))
                    .allow_scroll(true);
    
                
                let color = if ui.ctx().style().visuals.dark_mode {
                    // check if the ui is in dark mode. 
                    // Light blue looks nice on dark mode but hard to see in light mode.
                    egui::Color32::LIGHT_BLUE
                } else {
                    egui::Color32::BLACK
                };
    
                plot.show(ui, |plot_ui| { 
                    let plot_min_x = plot_ui.plot_bounds().min()[0];
                    let plot_max_x = plot_ui.plot_bounds().max()[0];

                    if let Some(step_line) = self.histogrammer.egui_histogram_step(&hist_name, color) {
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
    
                    fit_handler.markers.cursor_position = plot_ui.pointer_coordinate();
                    fit_handler.markers.draw_markers(plot_ui);
                    fit_handler.draw_fits(plot_ui);

                });
            }
        
        }

    }

    pub fn render_2d_histogram(&mut self, ui: &mut egui::Ui) {
        if let Some(hist_name) = self.selected_histograms.first() {
            if let Some(HistogramTypes::Hist2D(hist)) = self.histogrammer.histogram_list.get(hist_name.as_str()) {
                
                // cut handler ui
                self.cut_handler.cut_handler_ui(ui);

                let plot = Plot::new(&hist_name)
                    .legend(Legend::default())
                    .clamp_grid(true)
                    .allow_drag(false)
                    .allow_zoom(false)
                    .allow_boxed_zoom(true)
                    .auto_bounds(egui::Vec2b::new(true, true))
                    .allow_scroll(true);

                let color = if ui.ctx().style().visuals.dark_mode {
                    egui::Color32::LIGHT_BLUE
                } else {
                    egui::Color32::DARK_BLUE
                };

                plot.show(ui, |plot_ui| { 
                    let plot_min_x = plot_ui.plot_bounds().min()[0];
                    let plot_max_x = plot_ui.plot_bounds().max()[0];
                    let plot_min_y = plot_ui.plot_bounds().min()[1];
                    let plot_max_y = plot_ui.plot_bounds().max()[1];

                    if let Some(bar_chart) = self.histogrammer.egui_heatmap(&hist_name) {
                        plot_ui.bar_chart(bar_chart.color(color));

                        let stats_entries = hist.legend_entries(plot_min_x, plot_max_x, plot_min_y, plot_max_y);

                        for (_i, entry) in stats_entries.iter().enumerate() {
                            plot_ui.text(
                                Text::new(PlotPoint::new(0, 0), " ") // Placeholder for positioning; adjust as needed
                                    .highlight(false)
                                    .color(color)
                                    .name(entry)
                            );
                        }
                    }

                    if self.cut_handler.draw_flag {
                        self.cut_handler.draw_active_cut(plot_ui);
                    }

                });
            }
        }
    }

    pub fn render_multiple_histograms(&mut self, ui: &mut egui::Ui) {

        // Set up the plot for the combined histogram display.
        let plot = Plot::new("Combined Histogram")
            .legend(Legend::default())
            .clamp_grid(true)
            .allow_drag(false)
            .allow_zoom(false)
            .allow_boxed_zoom(true)
            .auto_bounds(egui::Vec2b::new(true, true))
            .allow_scroll(true);

        // Display the plot in the UI.
        plot.show(ui, |plot_ui| {
            
            // Define a set of colors for the histograms.
            let colors: [egui::Color32; 5] = [
                egui::Color32::LIGHT_BLUE, 
                egui::Color32::LIGHT_RED, 
                egui::Color32::LIGHT_GREEN, 
                egui::Color32::LIGHT_YELLOW, 
                egui::Color32::LIGHT_GRAY
            ];

            let plot_min_x = plot_ui.plot_bounds().min()[0];
            let plot_max_x = plot_ui.plot_bounds().max()[0];
            let plot_min_y = plot_ui.plot_bounds().min()[1];
            let plot_max_y = plot_ui.plot_bounds().max()[1];

            for (i, selected_name) in self.selected_histograms.iter().enumerate() {
                // Render the appropriate histogram type based on its type.
                match self.histogrammer.get_histogram_type(selected_name) {
                    Some(HistogramTypes::Hist1D(hist)) => {

                        let hist_color = colors[i % colors.len()];
                        if let Some(step_line) = self.histogrammer.egui_histogram_step(selected_name, hist_color) {

                            plot_ui.line(step_line);

                            let stats_entries = hist.legend_entries(plot_min_x, plot_max_x);

                            for (_i, entry) in stats_entries.iter().enumerate() {
                                plot_ui.text(
                                    Text::new(PlotPoint::new(0, 0), " ") // Placeholder for positioning; adjust as needed
                                        .highlight(false)
                                        .color(hist_color)
                                        .name(entry)
                                );
                            }

                        }
                    }
                    Some(HistogramTypes::Hist2D(hist)) => {
                        
                        let hist_color = colors[i % colors.len()];

                        if let Some(bar_chart) = self.histogrammer.egui_heatmap(selected_name) {
                            plot_ui.bar_chart(bar_chart.color(hist_color));

                            let stats_entries = hist.legend_entries(plot_min_x, plot_max_x, plot_min_y, plot_max_y);

                            for (_i, entry) in stats_entries.iter().enumerate() {
                                plot_ui.text(
                                    Text::new(PlotPoint::new(0, 0), " ") // Placeholder for positioning; adjust as needed
                                        .highlight(false)
                                        .color(hist_color)
                                        .name(entry)
                                );
                            }

                        }
                    }

                    None => {
                        // Optionally handle the case where the histogram is not found or its type is not supported.
                        // ui.label(format!("Histogram '{}' not found or type not supported.", selected_name));
                    }
                }
            }
        });
    }

    pub fn render_histos(&mut self, ui: &mut egui::Ui) {
        if self.selected_histograms.is_empty() {
            ui.label("No histograms are selected");  
            return;        
        }
        if self.selected_histograms.len() == 1 {
            self.render_1d_histogram(ui);
            self.render_2d_histogram(ui);
        } else {
            self.render_multiple_histograms(ui);
        }
    }

    pub fn calculate_histograms(&mut self) {
        self.create_lazyframe();
        self.perform_histogrammer_from_lazyframe();
    }

    pub fn filter_lazyframe_with_cuts(&mut self) {

        // First, check if `self.lazyframer` is Some and get a mutable reference to it
        if let Some(ref mut lazyframer) = self.lazyframer {
            // Now you can access `lazyframer.lazyframe` because `lazyframer` is a mutable reference to `LazyFramer`
            if let Some(ref lazyframe) = lazyframer.lazyframe {
                match self.cut_handler.filter_lf_with_all_cuts(lazyframe) {
                    Ok(filtered_lf) => {
                        // Use the setter method to update the lazyframe
                        lazyframer.set_lazyframe(filtered_lf);
                        self.perform_histogrammer_from_lazyframe();
                    },
                    Err(e) => {
                        log::error!("Failed to filter LazyFrame with cuts: {}", e);
                    }
                }
            }
        }
    }

    pub fn save_current_lazyframe(&mut self) {
        // First, check if `self.lazyframer` is Some and get a mutable reference to it
        // if let Some(ref mut lazyframer) = self.lazyframer {
            // Now you can access `lazyframer.lazyframe` because `lazyframer` is a mutable reference to `LazyFramer`
                // Ask user for output file path
            if let Some(output_path) = rfd::FileDialog::new()
                .set_title("Collect Lazyframe and save the DataFrame to a single file")
                .add_filter("Parquet file", &["parquet"])
                .save_file() {

                if let Some(lazyframer) = &mut self.lazyframer {
                    match lazyframer.save_lazyframe(&output_path) {
                        Ok(_) => println!("LazyFrame saved successfully."),
                        Err(e) => log::error!("Failed to save LazyFrame: {}", e),
                    }
                } else {
                    log::error!("No LazyFrame loaded to save.");
                }
            }
        // }
    }

    pub fn calculation_ui(&mut self, ui: &mut egui::Ui) {
        ui.separator();

        ui.horizontal(|ui| {

            
            if ui.button("Calculate Histograms").clicked() {
                self.calculate_histograms();
            }

            // check to see if there is a lazyframe to cut
            if self.lazyframer.is_some() {
                // check to see if there are cuts

                ui.separator();

                if ui.button("Save Lazyframe").on_hover_text("CAUTION: The collected lazyframe must fit it memory\nThis saves the current lazyframe. It is advised to filter the lazyframe with cuts.").clicked() {
                    self.save_current_lazyframe();
                }

                if !self.cut_handler.cuts.is_empty() {

                    ui.separator();
                    if ui.button("Filter with Cuts").on_hover_text("CAUTION: The collected lazyframe must fit it memory").clicked() {
                        self.filter_lazyframe_with_cuts();
                    }

                    

                }

            }

        });

        ui.separator();

    }
}
