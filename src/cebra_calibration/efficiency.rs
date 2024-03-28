
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct GammaLine {
    energy: f64, // keV
    energy_uncertainty: f64, // keV
    intensity: f64, 
    intensity_uncertainty: f64,
    measured_counts: f64,
    measured_counts_uncertainty: f64,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct SourceActivity {
    activity: f64, // Bq
    date: Option<chrono::NaiveDate>,
}


#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct GammaSource {
    name: String,
    gamma_lines: Vec<GammaLine>,
    half_life: f64, // years
    source_activity_calibration: SourceActivity,
    source_activity_measurement: SourceActivity,
}

impl GammaSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calculate_source_activity_for_measurement(&mut self) {
        let calibration_date = self.source_activity_calibration.date.unwrap();
        let measurement_date = self.source_activity_measurement.date.unwrap();
        let half_life_years = self.half_life;
        let half_life_days = half_life_years * 365.25; // convert years to days

        let time_difference = measurement_date.signed_duration_since(calibration_date).num_days() as f64;
        let decay_constant = 0.693 / half_life_days;
        let activity = self.source_activity_calibration.activity * (-decay_constant * time_difference).exp();

        self.source_activity_measurement.activity = activity;
    }

    pub fn source_ui(&mut self, ui: &mut egui::Ui) {

        egui::Grid::new("source_ui")
            .num_columns(6)
            .spacing(egui::vec2(8.0, 4.0))
            .striped(true)
            .min_col_width(50.0)
            .show(ui, |ui| {

                ui.label("Source");
                ui.add(egui::TextEdit::singleline(&mut self.name));
                
                ui.label("Half-life:");
                ui.add(
                    egui::DragValue::new(&mut self.half_life)
                    .speed(0.1)
                    .clamp_range(0.0..=f64::INFINITY)
                    .suffix(" years")
                );

            ui.end_row();

                ui.label("Calibration");

                ui.label("Date:");

                let calibration_date = self.source_activity_calibration.date.get_or_insert_with(|| chrono::offset::Utc::now().date_naive());
                ui.add(
                    egui_extras::DatePickerButton::new(calibration_date)
                    .id_source("calibration_date")
                    .highlight_weekends(false)
                );

                ui.label("Activity:");
                ui.add(
                    egui::DragValue::new(&mut self.source_activity_calibration.activity)
                    .speed(1000.0)
                    .clamp_range(0.0..=f64::INFINITY)
                    .suffix(" Bq")
                );

            ui.end_row();

                ui.label("Measurement");

                ui.label("Date:");

                let measurement_date = self.source_activity_measurement.date.get_or_insert_with(|| chrono::offset::Utc::now().date_naive());
                ui.add(
                    egui_extras::DatePickerButton::new(measurement_date)
                    .id_source("measurement_date")
                    .highlight_weekends(false)
                );

                ui.label("Activity:");

                ui.label(&format!("{:.0} Bq", self.source_activity_measurement.activity));

                if ui.button("Calculate Activity").clicked() {
                    self.calculate_source_activity_for_measurement();
                }

            ui.end_row();



        });

        // ui.horizontal(|ui: &mut egui::Ui| {
        //     ui.label("Measurement Date");
        //     let measurement_date = self.source_activity.measurement_date.get_or_insert_with(|| chrono::offset::Utc::now().date_naive());
        //     ui.add(
        //         egui_extras::DatePickerButton::new(measurement_date)
        //         .id_source("measurement_date")
        //         .highlight_weekends(false)
        //     );
        // });

        // ui.label("Gamma Lines");
        // for gamma_line in &mut self.gamma_lines {
        //     gamma_line.gamma_line_ui(ui);
        // }
    }
}
