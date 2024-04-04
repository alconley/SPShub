use super::gamma_source::GammaSource;

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct DetectorLine {
    pub count: f64,
    pub uncertainty: f64,
    pub energy: f64,
    pub intensity: f64,
    pub intensity_uncertainty: f64,
    pub efficiency: f64,
    pub efficiency_uncertainty: f64,
}

impl DetectorLine {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::DragValue::new(&mut self.count).speed(1.0).clamp_range(0.0..=f64::INFINITY));
        ui.add(egui::DragValue::new(&mut self.uncertainty).speed(1.0).clamp_range(0.0..=f64::INFINITY));

        ui.label( format!("{:.2} ± {:.2}%", self.efficiency, self.efficiency_uncertainty) );
    }
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct Detector {
    pub name: String,
    pub lines: Vec<DetectorLine>,
}

impl Detector {
    pub fn ui(&mut self, ui: &mut egui::Ui, gamma_source: &GammaSource) {

        ui.horizontal(|ui| {
            ui.label("Detector Name:");
            ui.text_edit_singleline(&mut self.name);
        });

        ui.collapsing(self.name.to_string(), |ui| {

            let gamma_lines = gamma_source.gamma_lines.iter().map(|line| format!("{:.1} keV", line.energy)).collect::<Vec<_>>();

            egui::Grid::new("detector_grid")
                .striped(false)
                .num_columns(4)
                .show(ui, |ui| {
                    ui.label("Energy");
                    ui.label("Counts");
                    ui.label("Uncertainty");
                    ui.end_row();

                    let mut index_to_remove = None;
                    for (index, line) in self.lines.iter_mut().enumerate() {
                        egui::ComboBox::from_id_source(format!("Line {}", index))
                            .selected_text(format!("{:.1} keV", line.energy))
                            .show_ui(ui, |ui| {
                                for (gamma_index, gamma_line_str) in gamma_lines.iter().enumerate() {
                                    if ui.selectable_label(line.energy == gamma_source.gamma_lines[gamma_index].energy, gamma_line_str).clicked() {
                                        line.energy = gamma_source.gamma_lines[gamma_index].energy;
                                        line.intensity = gamma_source.gamma_lines[gamma_index].intensity;
                                        line.intensity_uncertainty = gamma_source.gamma_lines[gamma_index].intensity_uncertainty;
                                    }
                                }
                            });
        
                        line.ui(ui);
        
                        if ui.button("X").clicked() {
                            index_to_remove = Some(index);
                        }
        
                        ui.end_row();
                    }

                    if let Some(index) = index_to_remove {
                        self.remove_line(index);
                    }
                });

            if ui.button("+").clicked() {
                self.lines.push(DetectorLine::default());
            }

            for line in &mut self.lines {
                gamma_source.gamma_line_efficiency_from_source_measurement(line);
            }

        });


    }

    fn remove_line(&mut self, index: usize) {
        self.lines.remove(index);
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
            gamma_source: source.unwrap_or_default(),
            detectors: vec![],
        }
    }



    pub fn measurement_ui(&mut self, ui: &mut egui::Ui) {

        ui.collapsing("Measurement", |ui: &mut egui::Ui| {

            // ensure that there are gamma lines to display
            if self.gamma_source.gamma_lines.is_empty() {
                ui.label("No gamma lines added to source");
                return;
            }

            let mut index_to_remove = None;

            for (index, detector) in &mut self.detectors.iter_mut().enumerate() {

                detector.ui(ui, &self.gamma_source);

                if ui.button("Remove Detector").clicked() {
                    index_to_remove = Some(index);
                }

            }

            ui.separator();

            if ui.button("Add Detector").clicked() {
                self.detectors.push(Detector::default());
            }

            if let Some(index) = index_to_remove {
                self.detectors.remove(index);
            }

            ui.separator();

        });
            
    }

    pub fn update_ui(&mut self, ui: &mut egui::Ui) {

        ui.collapsing(format!("{} Measurement", self.gamma_source.name), |ui| {

            self.gamma_source.source_ui(ui);
            self.measurement_ui(ui);

        });
    }

}
