use log::info;

use eframe::egui::{self};
use eframe::App;

use super::histogrammer::histogram_script::add_histograms;
use super::workspacer::Workspace;
use super::lazyframer::LazyFramer;
use super::processer::Processer;

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
    processer: Processer,

    #[serde(skip)]
    lazyframer: Option<LazyFramer>,

    flags: PlotterAppFlags,

}

impl PlotterApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {

        Self {
            workspace: Workspace::new(),
            processer: Processer::new(),
            lazyframer: None,
            flags: PlotterAppFlags::default(),
        }
    }

    fn create_lazyframe_from_selected_files(&mut self) {

        log::info!("Creating LazyFrame from selected files");

        if !self.workspace.selected_files.is_empty() {

            self.lazyframer = Some(LazyFramer::new(self.workspace.selected_files.clone()));
            self.flags.lazyframe_loaded = true;
            
            // // Update CutHandler with column names from LazyFramer
            // if let Some(ref lazyframer) = self.lazyframer {
            //     let column_names = lazyframer.get_column_names();
            //     self.cut_handler.update_column_names(column_names);
            //     log::info!("Column names: {:?}", self.cut_handler.column_names.clone());
            // }
        }
    }

    fn perform_histogrammer_from_lazyframe(&mut self) {
        if let Some(lazyframer) = &self.lazyframer {
            if let Some(lf) = &lazyframer.lazyframe {
                match add_histograms(lf.clone()) { 
                    Ok(h) => {
                        // self.histogrammer = h; 
                        self.processer.histogrammer = h;
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
    
    /* 
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
    }*/

}


impl App for PlotterApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {

        let mut size = ctx.screen_rect().size();
        size.x -= 50.0; // Subtract 50 from the width
        size.y -= 100.0; // Subtract 50 from the height

        egui::Window::new("Plotter").max_size(size).show(ctx, |ui| {


            egui::TopBottomPanel::top("plotter_top_panel").show_inside(ui, |ui| {

                egui::menu::bar(ui, |ui| {

                    ui.menu_button("Workspace", |ui| {

                        // self.workspace.select_directory_ui(ui);
                        // self.workspace.file_selection_settings_ui(ui);
                        // self.workspace.file_selection_ui_in_menu(ui);

                        self.workspace.workspace_ui(ui);
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

                // if self.flags.show_cutter {
                //     ui.separator();
                //     // self.cutter_ui(ui);
                // } else {
                //     self.cut_handler.draw_flag = false;
                // }

            });

            // egui::TopBottomPanel::bottom("plotter_bottom_panel").resizable(true).show_inside(ui, |ui| {
                // self.fitter.interactive_keybinds(ui);
            // });

            egui::SidePanel::right("plotter_right_panel").show_inside(ui, |ui| {
                self.processer.select_histograms_ui(ui);
            });

            if self.workspace.file_selecton {
                egui::SidePanel::left("plotter_left_panel").show_inside(ui, |ui| {
                    self.workspace.file_selection_ui_side_panel(ui);
                });
            }

            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.processer.render_histos(ui);
                
            });

        });

    }

    
}



