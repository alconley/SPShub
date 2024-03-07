use egui::Vec2b;
use log::info;

use eframe::egui::{self, Color32};
use eframe::App;
use egui_plot::{Plot, Legend, Text, PlotPoint};

use std::path::PathBuf;
use std::sync::Arc;

use polars::prelude::*;

use super::histogram_creation::add_histograms;
use super::histogrammer::{Histogrammer, HistogramTypes};
use super::cut::CutHandler;
use super::workspace::Workspace;
use super::lazyframer::LazyFramer;
use super::fitter::Fit;  

// Flags to keep track of the state of the app
#[derive(serde::Deserialize, serde::Serialize)]
struct PlotterAppFlags {
    lazyframe_loaded: bool,
    histograms_loaded: bool,
    files_selected: bool,
    show_cutter: bool,
    cutter_cuts_exist: bool,
    cutter_save_to_one_file: bool,
    cutter_save_to_separate_files: bool,
    cutter_filter_lazyframe: bool,
    can_cut_lazyframe: bool,
}

impl Default for PlotterAppFlags {
    fn default() -> Self {
        Self {
            lazyframe_loaded: false,
            files_selected: false,
            histograms_loaded: false,
            show_cutter: false,
            cutter_cuts_exist: false,
            cutter_save_to_one_file: false,
            cutter_save_to_separate_files: false,
            cutter_filter_lazyframe: false,
            can_cut_lazyframe: false,
        }
    }
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct PlotterApp {
    workspace: Workspace,
    histogrammer: Histogrammer,
    fitter: Fit,

    cut_handler: CutHandler,
    selected_cut_id: Option<String>,

    #[serde(skip)]
    lazyframer: Option<LazyFramer>,

    #[serde(skip)]
    selected_histograms: Vec<String>,

    flags: PlotterAppFlags,

}

impl PlotterApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {

        Self {
            workspace: Workspace::new(),
            histogrammer: Histogrammer::new(),
            fitter: Fit::new(),
            cut_handler: CutHandler::new(), // have to update column_names with the columns from the lazyframe
            selected_cut_id: None,
            lazyframer: None,
            selected_histograms: Vec::new(),
            flags: PlotterAppFlags::default(),
        }
    }

    fn create_lazyframe_from_selected_files(&mut self) {

        log::info!("Creating LazyFrame from selected files");

        if !self.workspace.selected_files.is_empty() {

            let files_arc: Arc<[PathBuf]> = Arc::from(self.workspace.selected_files.clone());

            let args = ScanArgsParquet::default();
            log::info!("Files {:?}", files_arc);
            // Instead of using `?`, use a match or if let to handle the Result
            match LazyFrame::scan_parquet_files(files_arc, args) {
                Ok(lf) => {
                    // Successfully loaded LazyFrame, do something with it
                    // self.lazyframe = Some(lf);
                    self.lazyframer = Some(LazyFramer::new(lf));
                    self.flags.lazyframe_loaded = true;

                    // Update CutHandler with column names from LazyFramer
                    if let Some(ref lazyframer) = self.lazyframer {
                        let column_names = lazyframer.get_column_names();
                        self.cut_handler.update_column_names(column_names);

                        log::info!("Column names: {:?}", self.cut_handler.column_names.clone());
                    }
                },
                Err(e) => {
                    // Handle the error, e.g., log it
                    log::error!("Failed to load Parquet files: {}", e);
                }
            }
        }
    }

    fn perform_histogrammer_from_lazyframe(&mut self) {
        if let Some(lazyframer) = &self.lazyframer {
            match add_histograms(lazyframer.get_lazyframe().clone()) { 
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

    fn render_selected_histograms(&mut self, ui: &mut egui::Ui) {
                
        // Display a message if no histograms are selected.
        if self.selected_histograms.is_empty() {
            ui.label("No histogram selected");
            return;
        }


        // add the first histogram to the fitter
        if let Some(selected_name) = self.selected_histograms.first() {
            if let Some(histogram_type) = self.get_histogram_type(selected_name) {
                match histogram_type {
                    HistogramTypes::Hist1D(hist) => {
                        self.fitter.histogram = Some(hist.clone());
                    },
                    _ => {}
                }
            }
        }

        // Set up the plot for the combined histogram display.
        let plot = Plot::new("Combined Histogram")
            .legend(Legend::default())
            .clamp_grid(true)
            .allow_drag(false)
            .allow_zoom(false)
            .allow_boxed_zoom(true)
            .auto_bounds(Vec2b::new(true, true))
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

            if self.cut_handler.draw_flag {
                self.cut_handler.draw_active_cut(plot_ui);
            }

            self.fitter.markers.cursor_position = plot_ui.pointer_coordinate();
            self.fitter.markers.draw_markers(plot_ui);
            self.fitter.draw_fit_lines(plot_ui);


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

    fn cutter_ui(&mut self, ui: &mut egui::Ui) {

        ui.horizontal(|ui| {
        
            if ui.button("New Cut").clicked() {
                self.cut_handler.add_new_cut();
            }

            ui.separator();

            // Button for saving filtered data to a single file
            if ui.button("Save to One File").clicked() {
                self.flags.cutter_save_to_one_file = true; // Set a flag to handle file dialog and saving outside UI code
            }

            // if the flag is set, show the file dialog and filter the data with cuts
            if self.flags.cutter_save_to_one_file {

                // Reset the flag immediately to prevent repeated triggers
                self.flags.cutter_save_to_one_file = false;

                // Ask user for output file path
                if let Some(output_path) = rfd::FileDialog::new()
                    .set_title("Save Reduced DataFrame to a Single File")
                    .add_filter("Parquet file", &["parquet"])
                    .save_file() {
                    // Attempt to save the filtered data to one file
                    if let Err(e) = self.cut_handler.filter_files_and_save_to_one_file(self.workspace.selected_files.clone(), &output_path) {
                        eprintln!("Error saving to one file: {:?}", e);
                    }
                }
            }

            // Button for saving filtered data to separate files
            if ui.button("Save Separately").clicked() {
                self.flags.cutter_save_to_separate_files = true; // Set a flag to handle file dialog and saving outside UI code
            }

            if self.flags.cutter_save_to_separate_files {
                // Reset the flag immediately to prevent repeated triggers
                self.flags.cutter_save_to_separate_files = false;
        
                // Ask user for output directory
                if let Some(output_dir) = rfd::FileDialog::new()
                    .set_title("Select Directory to Save Each DataFrame Separately")
                    .pick_folder() {
                    // Prompt user for custom text to append to filenames
                    let custom_text = "filtered"; // This could be dynamically set based on user input
    
                    // Attempt to save the filtered data to separate files
                    if let Err(e) = self.cut_handler.filter_files_and_save_separately(self.workspace.selected_files.clone(), &output_dir, &custom_text.to_string()) {
                        eprintln!("Error saving files separately: {:?}", e);
                    }
                }
            }

            ui.separator();

            // filter the lazyframe with all cuts if the flag is triggered
            if self.flags.cutter_filter_lazyframe {

                self.flags.cutter_filter_lazyframe = false;

                //check if lazyframer is Some, and use that to enable or disable the button
                if let Some(ref lazyframer) = self.lazyframer {
                    // Now you have `lazyframer` which is a `&LazyFramer`, and you can call `get_lazyframe()` on it
                    match self.cut_handler.filter_lf_with_all_cuts(&lazyframer.get_lazyframe()) {
                        Ok(filtered_lf) => {
                            // Update self.lazyframe with the filtered LazyFrame
                            self.lazyframer = Some(LazyFramer::new(filtered_lf));
                            self.flags.lazyframe_loaded = true;

                            self.perform_histogrammer_from_lazyframe();
                        },
                        Err(e) => {
                            // Handle the error, e.g., log the error
                            log::error!("Failed to filter LazyFrame with cuts: {}", e);
                        }
                    }
                }

            }

        });

        ui.separator();


        // Iterate through all cuts to display selectable labels
        egui::Grid::new("cut_selector_grid").show(ui, |ui| {
            for (id, _cut) in self.cut_handler.cuts.iter_mut() {
                // Create a selectable label for the cut
                let is_selected = self.selected_cut_id.as_ref() == Some(id);
                if ui.selectable_label(is_selected, format!("Cut: {}", id)).clicked() {
                    // When a label is clicked, set this cut as the selected cut
                    self.selected_cut_id = Some(id.clone());
                    self.cut_handler.active_cut_id = Some(id.clone());
                }
            } 
        }); 


        // Display UI for the selected cut
        if let Some(selected_id) = &self.selected_cut_id {
            if let Some(selected_cut) = self.cut_handler.cuts.get_mut(selected_id) {

                ui.label("Selected Cut:");
                selected_cut.cut_ui(ui); // Display the `cut_ui` of the selected cut
                
                
                ui.separator();

                // check box to draw/edit the cut
                ui.checkbox(&mut self.cut_handler.draw_flag, "Draw");
                
                ui.separator();

                // button to remove cut
                if ui.button("Remove Cut").clicked() {
                    self.cut_handler.cuts.remove(selected_id);
                    self.selected_cut_id = None;
                }
            }
        }
    }

}


impl App for PlotterApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {

        let mut size = ctx.screen_rect().size();
        size.x -= 50.0; // Subtract 50 from the width
        size.y -= 100.0; // Subtract 50 from the height

        egui::Window::new("Plotter").max_size(size).show(ctx, |ui| {


            egui::TopBottomPanel::top("plotter_top_panel").show_inside(ui, |ui| {

                egui::menu::bar(ui, |ui| {

                    ui.menu_button("Status", |ui| {
                        ui.label(format!("LazyFrame loaded: {}", self.flags.lazyframe_loaded));
                        ui.label(format!("Histograms loaded: {}", self.flags.histograms_loaded));
                        ui.label(format!("Files selected: {}", self.workspace.selected_files.len()));
                        ui.label(format!("Show Cutter: {}", self.flags.show_cutter));
                    });

                    ui.separator();

                    ui.menu_button("Workspace", |ui| {

                        self.workspace.select_directory_ui(ui);
                        self.workspace.file_selection_settings_ui(ui);
                        self.workspace.file_selection_ui_in_menu(ui);

                    });


                    ui.separator();

                    if ui.button("Calculate histograms").clicked() {
                        self.flags.histograms_loaded = false;

                        self.create_lazyframe_from_selected_files();
        
                        info!("Calculating histograms");
        
                        self.perform_histogrammer_from_lazyframe();
                        self.flags.histograms_loaded = true;
        
                        info!("Finished caluclating histograms");
                    }


                    ui.separator();

                    // Checkbox to toggle the visibility of the cutter UI
                    ui.checkbox(&mut self.flags.show_cutter, "Cut Handler");

                    if self.flags.show_cutter {
                        if ui.button("Filter LazyFrame").clicked() {
                            self.flags.cutter_filter_lazyframe = true;
                        }
                    }

                    

                });


                if self.flags.show_cutter {
                    ui.separator();
                    self.cutter_ui(ui);
                } else {
                    self.cut_handler.draw_flag = false;
                }

            });


            if self.workspace.file_selecton {
                egui::SidePanel::left("plotter_left_panel").show_inside(ui, |ui| {
                    self.workspace.file_selection_ui_side_panel(ui);
                });
            }

            egui::SidePanel::right("plotter_right_panel").show_inside(ui, |ui| {

                self.histogram_buttons_ui(ui);

            });



            egui::TopBottomPanel::bottom("plotter_bottom_panel").show_inside(ui, |ui| {
                
                self.fitter.interactive_fitter(ui);

            });

            egui::CentralPanel::default().show_inside(ui, |ui| {

                self.render_selected_histograms(ui);
            });

        });

    }

    
}



