use log::info;

use eframe::egui::{self, Color32};
use eframe::App;
use egui_plot::{Plot, Legend, Text, PlotPoint};

use std::path::PathBuf;
use std::sync::Arc;

use polars::prelude::*;

use super::histogram_creation::add_histograms;
use super::histogrammer::{Histogrammer, HistogramTypes};

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct PlotterApp {
    files: Option<Vec<PathBuf>>,
    histogrammer: Histogrammer,

    // skip the serialize
    #[serde(skip)]
    lazyframe: Option<LazyFrame>,

    #[serde(skip)]
    lazyframe_columns: Vec<String>,

    histograms_loaded: bool,

    #[serde(skip)]
    selected_histograms: Vec<String>,
}

impl PlotterApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {

        Self {
            files: None,
            histogrammer: Histogrammer::new(),
            lazyframe: None,
            lazyframe_columns: Vec::new(),
            histograms_loaded: false,
            selected_histograms: Vec::new(),
        }
    }

    fn create_lazyframe_from_files(&mut self) {
        if let Some(files) = &self.files {
            let files_arc: Arc<[PathBuf]> = Arc::from(files.clone());
            let args = ScanArgsParquet::default();

            // Instead of using `?`, use a match or if let to handle the Result
            match LazyFrame::scan_parquet_files(files_arc, args) {
                Ok(lf) => {
                    // Successfully loaded LazyFrame, do something with it
                    self.lazyframe = Some(lf);
                },
                Err(e) => {
                    // Handle the error, e.g., log it
                    log::error!("Failed to load Parquet files: {}", e);
                }
            }
        }
    }

    fn perform_histogrammer_from_lazyframe(&mut self) {
        if let Some(lf) = &self.lazyframe {
            // dont like the cloning here... figure out later
            match add_histograms(lf.clone()) {
                Ok(h) => {
                    self.histogrammer = h;
                },
                Err(e) => {
                    log::error!("Failed to create histograms: {}", e);
                }
            }
        }
    }

    fn get_histogram_list(&self) -> Vec<String> {
        // Retrieves a sorted list of histogram names.
        let mut histogram_names: Vec<String> = self.histogrammer.histogram_list
            .keys()
            .cloned()
            .collect();
        histogram_names.sort();
        histogram_names
    }

    fn get_histogram_type(&self, name: &str) -> Option<&HistogramTypes> {
        self.histogrammer.histogram_list.get(name)
    }

    pub fn render_selected_histograms(&mut self, ui: &mut egui::Ui) {
        // Display a message if no histograms are selected.
        if self.selected_histograms.is_empty() {
            ui.label("No histogram selected");
            return;
        }

        // Set up the plot for the combined histogram display.
        let plot = Plot::new("Combined Histogram")
            .legend(Legend::default())
            .clamp_grid(true)
            .allow_drag(false)
            .allow_zoom(false)
            .allow_boxed_zoom(true)
            .allow_scroll(true);

        
        // Display the plot in the UI.
        plot.show(ui, |plot_ui| {

            // Define a set of colors for the histograms.
            let colors: [Color32; 5] = [
                Color32::LIGHT_BLUE, 
                Color32::LIGHT_RED, 
                Color32::LIGHT_GREEN, 
                Color32::LIGHT_YELLOW, 
                Color32::LIGHT_GRAY
            ];

            let plot_min_x = plot_ui.plot_bounds().min()[0];
            let plot_max_x = plot_ui.plot_bounds().max()[0];
            let plot_min_y = plot_ui.plot_bounds().min()[1];
            let plot_max_y = plot_ui.plot_bounds().max()[1];

            for (i, selected_name) in self.selected_histograms.iter().enumerate() {
                // Render the appropriate histogram type based on its type.
                match self.get_histogram_type(selected_name) {
                    Some(HistogramTypes::Hist1D(hist)) => {

                        // Render a 1D histogram as a step line.
                        let hist_color = colors[i % colors.len()];
                        // if let Some(step_line) = self.histogrammer.egui_histogram_step(selected_name, colors[i % colors.len()]) {
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

                        // Render a 2D histogram as a heatmap.
                        if let Some(bar_chart) = self.histogrammer.egui_heatmap(selected_name) {
                            plot_ui.bar_chart(bar_chart);

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

    fn histogram_buttons_ui(&mut self, ui: &mut egui::Ui) {
        
        ui.label("Histograms"); // Label for the histogram buttons.
        
        let keys: Vec<String> = self.get_histogram_list(); // Retrieve the list of histogram names.

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

    fn file_ui(&mut self, ui: &mut egui::Ui) {

        egui::menu::bar(ui, |ui| {

            if ui.button("Files").clicked() {
                self.histograms_loaded = false;

                
                if let Some(files) = rfd::FileDialog::new().add_filter("Parquet files", &["parquet"]).pick_files() {
                        info!("Files: {:?}", files);
                        self.files = Some(files);
                }
            }

            ui.separator();

            if ui.button("Calculate histograms").clicked() {
                self.create_lazyframe_from_files();

                info!("Calculating histograms");

                self.perform_histogrammer_from_lazyframe();
                self.histograms_loaded = true;

                info!("Finished caluclating histograms");
            }

        });

    }

}


impl App for PlotterApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("Plotter").show(ctx, |ui| {

            egui::TopBottomPanel::top("plotter_top_panel").show_inside(ui, |ui| {

                self.file_ui(ui);

            });

            egui::SidePanel::right("plotter_right_panel").show_inside(ui, |ui| {

                self.histogram_buttons_ui(ui);

            });


            // egui::TopBottomPanel::bottom("plotter_bottom_panel").show_inside(ui, |ui| {
            //     // self.reaction_ui(ui);                
            
            // });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                // self.plot(ui);

                self.render_selected_histograms(ui);
            });

        });

    }

    
}



