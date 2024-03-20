use eframe::egui::{self};
use eframe::App;

use super::workspacer::Workspacer;
use super::processer::Processer;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct PlotterApp {
    workspacer: Workspacer,
    processer: Processer,
}

impl PlotterApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {

        Self {
            workspacer: Workspacer::new(),
            processer: Processer::new(),
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

                    ui.menu_button("Workspace", |ui| {
                        self.workspacer.workspace_ui(ui);
                    });

                    if self.workspacer.selected_files.len() > 0 {

                        self.processer.files = self.workspacer.selected_files.clone();
                        self.processer.calculation_ui(ui);
                        // ui.separator();

                        // if ui.button("Calculate histograms").clicked() {
                        //     // add selected files to processer
                        //     self.processer.files = self.workspacer.selected_files.clone();
                        //     info!("Calculating histograms");
    
                        //     self.processer.calculate_histograms();
                        //     info!("Finished caluclating histograms");
                        // }

                        // ui.separator();
                    }
                });
            });

            egui::SidePanel::right("plotter_right_panel").show_inside(ui, |ui| {
                self.processer.select_histograms_ui(ui);
            });

            if self.workspacer.file_selecton {
                egui::SidePanel::left("plotter_left_panel").show_inside(ui, |ui| {
                    self.workspacer.file_selection_ui_side_panel(ui);
                });
            }

            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.processer.render_histos(ui);
            });

        });

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