use eframe::egui::{self};
use eframe::App;

const CHARGE: f64 = 1.6e-19; // Elementary charge in C
const NA: f64 = 6.023e23; // Avogadro's number

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
struct RunTimeSettings {
    cross_section: f64,     // µb/sr
    target_density: f64,    // µg/cm^2
    target_molar_mass: f64, // g/mol
    beam_current: f64,      // nA
    z_beam: i32,            // proton number
    slit_settings: f64,     // msr
    desired_counts: i64,    // counts
    time_s: f64,            // seconds
    time_h: f64,            // hours
    time_d: f64,            // days
}

impl RunTimeSettings {
    pub fn new() -> Self {
        Self {
            cross_section: 100.0,
            target_density: 100.0,
            target_molar_mass: 240.0,
            beam_current: 20.0,
            z_beam: 1,
            slit_settings: 4.62,
            desired_counts: 1000,
            time_s: 0.0,
            time_h: 0.0,
            time_d: 0.0,
        }
    }

    fn calculate_beam_time(&mut self) {
        let slits_sr = self.slit_settings * 1e-3; // msr to sr
        let target_density = self.target_density * 1e-6; // µg/cm^2 to g/cm^2
        let beam_current = self.beam_current * 1e-9; // nA to A
        let f_target = (target_density * NA) / (self.target_molar_mass) * (1e-24) * (1e-6);

        let run_time_s = (self.z_beam as f64 * CHARGE * self.desired_counts as f64)
            / (self.cross_section * f_target * slits_sr * beam_current);

        self.time_s = run_time_s;
        self.time_h = run_time_s / 3600.0;
        self.time_d = self.time_h / 24.0;
    }
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct SPSRunTimeApp {
    settings: RunTimeSettings,
}

impl SPSRunTimeApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            settings: RunTimeSettings::new(),
        }
    }
}

impl App for SPSRunTimeApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("SE-SPS Run Time Estimator")
            .show(ctx, |ui| {

                // Checkbox to toggle between mb/sr and µb/sr
                egui::Grid::new("runtime_settings_grid")
                    .num_columns(2)
                    .spacing(egui::vec2(8.0, 4.0))
                    .striped(true)
                    .show(ui, |ui| {

                    ui.label("Cross Section:");

                        // Display and adjust the cross section as microbarns
                    ui.add(
                        egui::DragValue::new(&mut self.settings.cross_section)
                            .speed(1.0)
                            .suffix(" µb/sr")
                            .clamp_range(0.0..=f64::INFINITY)
                        );
                    ui.end_row();


                    ui.label("Target Density:");
                    ui.add(
                        egui::DragValue::new(&mut self.settings.target_density)
                            .speed(1.0)
                            .suffix(" µg/cm^2")
                            .clamp_range(0.0..=f64::INFINITY)
                    );
                    ui.end_row();

                    ui.label("Target Molar Mass:");
                    ui.add(
                        egui::DragValue::new(&mut self.settings.target_molar_mass)
                            .speed(1.0)
                            .suffix(" g/mol")
                            .clamp_range(0.0..=f64::INFINITY)
                    );
                    ui.end_row();

                    ui.label("Beam Current:");
                    ui.add(
                        egui::DragValue::new(&mut self.settings.beam_current)
                            .speed(1.0)
                            .suffix(" nA")
                            .clamp_range(0.0..=f64::INFINITY)
                    ).on_hover_text("Beam current on target.");
                    ui.end_row();

                    ui.label("Z Beam:");
                    ui.add(
                        egui::DragValue::new(&mut self.settings.z_beam)
                            .speed(1.0)
                            .prefix("Z = ")
                            .clamp_range(1..=118) // Adjusted to allow minimum 1
                    ).on_hover_text("Proton number of the beam.");
                    ui.end_row();

                    ui.label("Slit Settings:");
                    ui.add(
                        egui::DragValue::new(&mut self.settings.slit_settings)
                            .speed(0.1)
                            .suffix(" msr")
                            .clamp_range(0.0..=12.8)
                    ).on_hover_text("Solid angle of the SE-SPS. Typical value is 4.62 msr. The SE-SPS has a max solid angle of 12.8 msr.");
                    ui.end_row();

                    ui.label("Desired Counts:");
                    ui.add(
                        egui::DragValue::new(&mut self.settings.desired_counts)
                            .speed(1.0)
                            .suffix(" counts")
                            .clamp_range(0..=i64::MAX)
                    ).on_hover_text("The desired number of counts in the peak of interest.");
                    ui.end_row();

                    // Call calculate_beam_time here if it should happen automatically upon any change
                    self.settings.calculate_beam_time();
                    ui.label("Estimated Time:");
                    ui.label(format!("{:.0} s | {:.2} h | {:.2} d", self.settings.time_s, self.settings.time_h, self.settings.time_d));
                    ui.end_row();
                });

            });
    }
}
