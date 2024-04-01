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
    pub selected_gamma_line_index: usize,
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
                let default_name = format!("Detector {}", self.detectors.len());

                self.detectors.push(Detector {
                    name: default_name,
                    // Ensure there's at least one DetectorLine, regardless of gamma_lines being empty
                    lines: if self.gamma_source.gamma_lines.is_empty() {
                        vec![DetectorLine::default()]
                    } else {
                        vec![DetectorLine::default(); self.gamma_source.gamma_lines.len()]
                    },
                    selected_gamma_line_index: 0,
                });
            }

            // Displaying each detector for naming
            for detector in &mut self.detectors {
                ui.horizontal(|ui| {
                    ui.label("Detector Name:");
                    ui.text_edit_singleline(&mut detector.name);
                });
            }

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("detector_measurement_grid")
                    .striped(true)
                    .show(ui, |ui| {

                        ui.label(format!("Source: {}", self.gamma_source.name));
                        for i in 0..self.detectors.len() {
                            ui.label(format!("{}", self.detectors[i].name));
                        }
                        ui.end_row();

                        ui.label("Energy (keV)");
                        for _i in 0..self.detectors.len() {
                            ui.label("Counts/±");
                        }
                        ui.end_row();

                        for detector in &mut self.detectors {
                            ui.horizontal(|ui| {
                                ui.label("Detector Name:");
                                ui.text_edit_singleline(&mut detector.name);
                            });
                        }

                        ui.separator();
            

                        for detector in &mut self.detectors {

                            // Combo Box for selecting a gamma line
                            let gamma_line_choices = self.gamma_source.gamma_lines.iter().map(|line| format!("{:.1} keV", line.energy)).collect::<Vec<_>>();
                            if !gamma_line_choices.is_empty() {
                                egui::ComboBox::from_label("")
                                    .selected_text(gamma_line_choices[detector.selected_gamma_line_index].clone())
                                    .show_ui(ui, |ui| {
                                        for (index, choice) in gamma_line_choices.iter().enumerate() {
                                            ui.selectable_value(&mut detector.selected_gamma_line_index, index, choice);
                                        }
                                    });
                            } else {
                                ui.label("No gamma lines added to source");
                            }

                    
                        }
                        
                        ui.end_row();
                    });
                });
        });
            
    }

    pub fn update_ui(&mut self, ui: &mut egui::Ui) {

        ui.collapsing(format!("{} Measurement", self.gamma_source.name), |ui| {

            self.gamma_source.source_ui(ui);
            self.measurement_ui(ui);

        });
    }


}
