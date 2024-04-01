use eframe::egui::{self};
use eframe::App;

use rfd::FileDialog;
use serde_yaml;
use std::fs::File;
use std::io::{Read, Write};

use super::detector::Measurement;
use super::gamma_source::GammaSource;


#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct CeBrAEfficiencyApp {
    gamma_sources: Vec<GammaSource>,
    measurements: Vec<Measurement>,
}

impl CeBrAEfficiencyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            gamma_sources: vec![],
            measurements: vec![],
        }
    }


    fn add_fsu_152eu_source(&mut self) {

        let mut gamma_source = GammaSource::new();
        gamma_source.name = "152Eu".to_string();
        gamma_source.half_life = 13.517; // years

        gamma_source.source_activity_calibration.activity = 74.370; // kBq
        gamma_source.source_activity_calibration.date = chrono::NaiveDate::from_ymd_opt(2017, 3, 17);

        gamma_source.add_gamma_line(121.7817, 28.53, 0.16);
        gamma_source.add_gamma_line(244.6974, 7.55, 0.04);
        gamma_source.add_gamma_line(344.2785, 26.59, 0.20);
        gamma_source.add_gamma_line(411.1164, 2.237, 0.013);
        gamma_source.add_gamma_line(443.9650, 2.827, 0.014);
        gamma_source.add_gamma_line(778.9045, 12.93, 0.08);
        gamma_source.add_gamma_line(867.3800, 4.23, 0.03);
        gamma_source.add_gamma_line(964.0570, 14.51, 0.07);
        gamma_source.add_gamma_line(1085.837, 10.11, 0.05);
        gamma_source.add_gamma_line(1112.076, 13.67, 0.08);
        gamma_source.add_gamma_line(1408.0130, 20.87, 0.09);

        self.gamma_sources.push(gamma_source);
    }

    fn add_fsu_56co_source(&mut self) {

        let mut gamma_source = GammaSource::new();
        gamma_source.name = "56Co".to_string();

        let co60_halflife_days = 77.236; // days
        gamma_source.half_life = co60_halflife_days/365.25; // years

        gamma_source.source_activity_calibration.activity = 108.0; // kBq (arbitrary scaled to match 152Eu)
        gamma_source.source_activity_calibration.date = chrono::NaiveDate::from_ymd_opt(2022, 4, 18);

        gamma_source.add_gamma_line(846.7638, 99.9399, 0.0023);
        gamma_source.add_gamma_line(1037.8333, 14.03, 0.05);
        gamma_source.add_gamma_line(1360.196, 4.283, 0.013);
        gamma_source.add_gamma_line(2598.438, 16.96, 0.04);
        gamma_source.add_gamma_line(3451.119, 0.942, 0.006);

        self.gamma_sources.push(gamma_source);
    }

    fn remove_gamma_source(&mut self, index: usize) {
        self.gamma_sources.remove(index);
    }

    fn save_to_file(&self) {
        if let Some(path) = FileDialog::new()
            .set_title("Save As")
            .add_filter("YAML", &["yaml", "yml"])
            .save_file() 
        {
            match File::create(path) {
                Ok(mut file) => {
                    let data = serde_yaml::to_string(self).expect("Failed to serialize data.");
                    file.write_all(data.as_bytes()).expect("Failed to write data to file.");
                }
                Err(e) => {
                    eprintln!("Failed to save file: {}", e);
                }
            }
        }
    }

    fn load_from_file() -> Self {
        if let Some(path) = FileDialog::new()
            .set_title("Open")
            .add_filter("YAML", &["yaml", "yml"])
            .pick_file() 
        {
            match File::open(path) {
                Ok(mut file) => {
                    let mut data = String::new();
                    file.read_to_string(&mut data).expect("Failed to read data from file.");
                    serde_yaml::from_str(&data).expect("Failed to deserialize data.")
                }
                Err(e) => {
                    eprintln!("Failed to load file: {}", e);
                    Self::default() // Return default if loading fails
                }
            }
        } else {
            Self::default() // Return default if no file is picked
        }
    }

}

impl App for CeBrAEfficiencyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("CeBrA Efficiency").show(ctx, |ui| {

            egui::TopBottomPanel::top("cebra_efficiency_top_panel").show_inside(ui, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Save").clicked() {
                            self.save_to_file();
                            ui.close_menu();
                        }
                        if ui.button("Load").clicked() {
                            *self = Self::load_from_file();
                            ui.close_menu();
                        }
                    });
                });
            });

            ui.horizontal(| ui| {
                ui.label("FSU's Current Sources:");

                // For "152Eu"
                let eu_source_index = self.gamma_sources.iter().position(|s| s.name == "152Eu");
                let eu_source_added = eu_source_index.is_some();
                if ui.selectable_label(eu_source_added, "152Eu").clicked() {
                    if let Some(index) = eu_source_index {
                        // Remove if it was already added
                        self.remove_gamma_source(index);
                    } else {
                        // Add if it was not added
                        self.add_fsu_152eu_source();
                    }
                }

                // For "56Co"
                let co_source_index = self.gamma_sources.iter().position(|s| s.name == "56Co");
                let co_source_added = co_source_index.is_some();
                if ui.selectable_label(co_source_added, "56Co").clicked() {
                    if let Some(index) = co_source_index {
                        // Remove if it was already added
                        self.remove_gamma_source(index);
                    } else {
                        // Add if it was not added
                        self.add_fsu_56co_source();
                    }
                }

                ui.separator();

                if ui.button("Add New γ Source").clicked() {
                    self.gamma_sources.push(GammaSource::new());
                }

                ui.separator();

            });

            ui.separator();
            
            let mut index_to_remove: Option<usize> = None;

            ui.label("Sources");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, gamma_source) in self.gamma_sources.iter_mut().enumerate() {

                    gamma_source.source_ui(ui);

                    // // add measurements button
                    // if ui.button("Add Measurement").clicked() {
                    //     self.measurements.push(Measurement::new(gamma_source.clone()));
                    // }

                    if gamma_source.name == "152Eu" || gamma_source.name == "56Co" {
                        continue;
                    } else {
                        if ui.button("Remove").clicked() {
                            index_to_remove = Some(index);
                        }
                    }



                }

                // for measurement in &mut self.measurements {
                //     measurement.update_ui(ui);
                // }

                if let Some(index) = index_to_remove {
                    self.remove_gamma_source(index);
                }
            });
        });
    }
}


