use super::exp_fitter::ExpFitter;
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
        ui.add(
            egui::DragValue::new(&mut self.count)
                .speed(1.0)
                .clamp_range(0.0..=f64::INFINITY),
        );
        ui.add(
            egui::DragValue::new(&mut self.uncertainty)
                .speed(1.0)
                .clamp_range(0.0..=f64::INFINITY),
        );

        ui.label(format!(
            "{:.2} ± {:.2}%",
            self.efficiency, self.efficiency_uncertainty
        ));
    }
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct Detector {
    pub name: String,
    pub lines: Vec<DetectorLine>,
    pub exp_fit: Option<ExpFitter>,
    pub to_remove: Option<bool>,
}

impl Detector {
    pub fn ui(&mut self, ui: &mut egui::Ui, gamma_source: &GammaSource) {
        ui.horizontal(|ui| {
            ui.label("Detector Name:");
            ui.text_edit_singleline(&mut self.name);

            if ui.button("X").clicked() {
                self.to_remove = Some(true);
            }
        });

        ui.collapsing(self.name.to_string(), |ui| {
            let gamma_lines = gamma_source
                .gamma_lines
                .iter()
                .map(|line| format!("{:.1} keV", line.energy))
                .collect::<Vec<_>>();

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
                                for (gamma_index, gamma_line_str) in gamma_lines.iter().enumerate()
                                {
                                    if ui
                                        .selectable_label(
                                            line.energy
                                                == gamma_source.gamma_lines[gamma_index].energy,
                                            gamma_line_str,
                                        )
                                        .clicked()
                                    {
                                        line.energy = gamma_source.gamma_lines[gamma_index].energy;
                                        line.intensity =
                                            gamma_source.gamma_lines[gamma_index].intensity;
                                        line.intensity_uncertainty = gamma_source.gamma_lines
                                            [gamma_index]
                                            .intensity_uncertainty;
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

            ui.horizontal(|ui| {
                if ui.button("+").clicked() {
                    self.lines.push(DetectorLine::default());
                }

                ui.separator();

                ui.label("Exponential Fitter:");

                let x_data = self
                    .lines
                    .iter()
                    .map(|line| line.energy)
                    .collect::<Vec<_>>();
                let y_data = self
                    .lines
                    .iter()
                    .map(|line| line.efficiency)
                    .collect::<Vec<_>>();
                let weights = self
                    .lines
                    .iter()
                    .map(|line| 1.0 / line.efficiency_uncertainty)
                    .collect::<Vec<_>>();

                if ui.button("Single").clicked() {
                    let mut exp_fit =
                        ExpFitter::new(x_data.clone(), y_data.clone(), weights.clone());
                    exp_fit.single_exp_fit();
                    self.exp_fit = Some(exp_fit);
                }

                if ui.button("Double").clicked() {
                    let mut exp_fit =
                        ExpFitter::new(x_data.clone(), y_data.clone(), weights.clone());
                    exp_fit.double_exp_fit();
                    self.exp_fit = Some(exp_fit);
                }

                ui.separator();

                if ui.button("Clear").clicked() {
                    if let Some(exp_fit) = &mut self.exp_fit {
                        exp_fit.fit_params = None;
                        exp_fit.fit_line = None;
                        exp_fit.fit_uncertainity_lines = None;
                        exp_fit.fit_label = "".to_string();
                    }
                }
            });

            if let Some(exp_fit) = &mut self.exp_fit {
                ui.label(exp_fit.fit_label.to_string());
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
