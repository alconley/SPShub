use super::gamma_source::GammaSource;

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct DetectorLine {
    pub count: f64,
    pub uncertainty: f64,
    pub gamma_line_energy: f64,
}

impl DetectorLine {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut self.count).speed(1.0).clamp_range(0.0..=f64::INFINITY));
            ui.add(egui::DragValue::new(&mut self.uncertainty).speed(1.0).clamp_range(0.0..=f64::INFINITY));
        });
    }
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct Detector {
    pub name: String,
    pub lines: Vec<DetectorLine>,
    pub available_gamma_lines: Vec<String>,
}

impl Detector {
    pub fn ui(&mut self, ui: &mut egui::Ui, gamma_source: &GammaSource) {
        ui.horizontal(|ui| {
            ui.label("Detector Name:");
            ui.text_edit_singleline(&mut self.name);
        });
        ui.separator();

        // Placeholder for user's gamma line selection
        let mut selected_gamma_line_energy = 0.0; // Default to the first gamma line's energy if applicable

        // Combo Box for selecting a gamma line
        egui::ComboBox::from_label("Select Gamma Line")
            .selected_text(format!("{:.1} keV", selected_gamma_line_energy))
            .show_ui(ui, |ui| {
                for gamma_line in &gamma_source.gamma_lines {
                    let line_text = format!("{:.1} keV", gamma_line.energy);
                    if ui.selectable_label(selected_gamma_line_energy == gamma_line.energy, line_text).clicked() {
                        selected_gamma_line_energy = gamma_line.energy;
                    }
                }
            });

        // Button to add a new DetectorLine with the selected gamma line's energy
        if ui.button("Add Detector Line").clicked() {
            self.lines.push(DetectorLine {
                count: 0.0,
                uncertainty: 0.0,
                gamma_line_energy: selected_gamma_line_energy,
            });
        }

        // Display each DetectorLine's UI
        for (index, line) in self.lines.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("Line {}", index + 1));
                line.ui(ui);
                if ui.button(format!("Remove {}", index + 1)).clicked() {
                    // Logic to mark this line for removal; actual removal happens after the loop
                }
            });
        }

        // Logic to remove marked lines, if any (not shown here due to Rust's borrowing rules)
    }
}


#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct Measurement {
    pub gamma_source: GammaSource,
    pub detectors: Vec<Detector>,
}


impl Measurement {
    pub fn new(source: Option<GammaSource>) -> Self {
        Self {
            gamma_source: source.unwrap_or(GammaSource::new()),
            detectors: vec![],
        }
    }

    pub fn measurement_ui(&mut self, ui: &mut egui::Ui) {

        ui.collapsing("Measurement", |ui: &mut egui::Ui| {

            // ensure that there are gamma lines to display
            if self.gamma_source.gamma_lines.len() == 0 {
                ui.label("No gamma lines added to source");
                return;
            }

            // Section for adding and naming detectors
            if ui.button("Add Detector").clicked() {
                self.detectors.push(Detector::default());
            }

            // Displaying each detector for naming
            for detector in &mut self.detectors {
                detector.ui(ui, &self.gamma_source);
            }

            ui.separator();

            // egui::ScrollArea::vertical().show(ui, |ui| {
            //     egui::Grid::new("detector_measurement_grid")
            //         .striped(true)
            //         .show(ui, |ui| {

            //             // ui.label(format!("Source: {}", self.gamma_source.name));
            //             // for i in 0..self.detectors.len() {
            //             //     ui.label(format!("{}", self.detectors[i].name));
            //             // }
            //             // ui.end_row();

            //             ui.label("Energy (keV)");
            //             for _i in 0..self.detectors.len() {
            //                 ui.label("Counts/±");
            //             }
            //             ui.end_row();
            
            //             for detector in &mut self.detectors {

            //                 // Combo Box for selecting a gamma line
            //                 let gamma_line_choices = self.gamma_source.gamma_lines.iter().map(|line| format!("{:.1} keV", line.energy)).collect::<Vec<_>>();
            //                 if !gamma_line_choices.is_empty() {
            //                     egui::ComboBox::from_label("")
            //                         .selected_text(gamma_line_choices[detector.selected_gamma_line_index].clone())
            //                         .show_ui(ui, |ui| {
            //                             for (index, choice) in gamma_line_choices.iter().enumerate() {
            //                                 ui.selectable_value(&mut detector.selected_gamma_line_index, index, choice);
            //                             }
            //                         });
            //                 } else {
            //                     ui.label("No gamma lines added to source");
            //                 }

                    
            //             }
                        
            //             ui.end_row();
            //         });
            //     });
        });
            
    }

    pub fn update_ui(&mut self, ui: &mut egui::Ui) {

        ui.collapsing(format!("{} Measurement", self.gamma_source.name), |ui| {

            self.gamma_source.source_ui(ui);
            self.measurement_ui(ui);

        });
    }


}
