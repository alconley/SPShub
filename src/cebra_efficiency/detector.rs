use super::gamma_source::GammaSource;

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct DetectorLine {
    pub count: f64,
    pub uncertainty: f64,
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct Detector {
    pub name: String,
    pub lines: Vec<DetectorLine>,
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct Measurement {
    pub gamma_source: GammaSource,
    pub detectors: Vec<Detector>,
}


impl Measurement {
    pub fn new() -> Self {
        Self {
            gamma_source: GammaSource::new(),
            detectors: vec![],
        }
    }
    // Add methods to manipulate detectors as needed...

    pub fn update_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Measurement");
        // Section for adding and naming detectors
        if ui.button("Add Detector").clicked() {
            let default_name = format!("Detector {}", self.detectors.len());
            self.detectors.push(Detector {
                name: default_name,
                lines: vec![DetectorLine::default(); self.gamma_source.gamma_lines.len()],
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

        // Setup for the grid
        egui::Grid::new("gamma_lines_grid")
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

                // Combo box for selecting gamma line energies
                let gamma_line_choices = self.gamma_source.gamma_lines.iter().map(|line| format!("{:.1}", line.energy)).collect::<Vec<_>>();
                let mut selected_gamma_line_index = 0; // Default to first gamma line
                egui::ComboBox::from_label("")
                    .selected_text(gamma_line_choices[selected_gamma_line_index].clone())
                    .show_ui(ui, |ui| {
                        for (index, choice) in gamma_line_choices.iter().enumerate() {
                            ui.selectable_value(&mut selected_gamma_line_index, index, choice);
                        }
                    });

                for detector in &mut self.detectors {
                    let line = &mut detector.lines[selected_gamma_line_index];
                    ui.horizontal(| ui| {
                        ui.add(egui::DragValue::new(&mut line.count).speed(1.0).clamp_range(0.0..=f64::INFINITY));
                        ui.add(egui::DragValue::new(&mut line.uncertainty).speed(1.0).clamp_range(0.0..=f64::INFINITY));
                    });

                }
                ui.end_row();
            });
    }
}
