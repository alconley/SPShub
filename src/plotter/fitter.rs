use egui_plot::{VLine, PlotUi, PlotPoint};
use egui::{Color32, Stroke};

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct EguiFitMarkers {
    pub region_markers: Vec<f64>,
    pub peak_markers: Vec<f64>,
    pub background_markers: Vec<f64>,

    #[serde(skip)]
    pub cursor_position: Option<PlotPoint>,
}

impl EguiFitMarkers {
    pub fn new() -> Self {
        Self {
            region_markers: Vec::new(),
            peak_markers: Vec::new(),
            background_markers: Vec::new(),
            cursor_position: None,
        }
    }

    pub fn add_region_marker(&mut self, x: f64) {
        self.region_markers.push(x);
    }

    pub fn sort_region_markers(&mut self) {
        self.region_markers.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }

    pub fn add_peak_marker(&mut self, x: f64) {
        self.peak_markers.push(x);
    }

    pub fn sort_peak_markers(&mut self) {
        self.peak_markers.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }

    pub fn add_background_marker(&mut self, x: f64) {
        self.background_markers.push(x);
    }

    pub fn sort_background_markers(&mut self) {
        self.background_markers.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }
    pub fn clear_region_markers(&mut self) {
        self.region_markers.clear();
    }

    pub fn clear_peak_markers(&mut self) {
        self.peak_markers.clear();
    }

    pub fn clear_background_markers(&mut self) {
        self.background_markers.clear();
    }

    fn draw_peak_markers(&mut self, plot_ui: &mut PlotUi) {
        for x in &self.peak_markers {
            let color = Color32::BLUE;
            let line = VLine::new(*x).color(color).stroke(Stroke::new(1.0, color));

            plot_ui.vline(line);
        }
    }

    fn draw_background_markers(&mut self, plot_ui: &mut PlotUi) {
        for x in &self.background_markers {
            let color = Color32::GREEN;
            let line = VLine::new(*x).color(color).stroke(Stroke::new(1.0, color));

            plot_ui.vline(line);
        }
    }

    fn draw_region_markers(&mut self, plot_ui: &mut PlotUi) {
        for x in &self.region_markers {
            let color = Color32::from_rgb(255, 0, 255);
            let line = VLine::new(*x).color(color).stroke(Stroke::new(1.0, color));

            plot_ui.vline(line);
        }
    }

    pub fn draw_markers(&mut self, plot_ui: &mut PlotUi) {
        self.draw_peak_markers(plot_ui);
        self.draw_background_markers(plot_ui);
        self.draw_region_markers(plot_ui);
    }

    pub fn interactive_markers(&mut self, ui: &mut egui::Ui) {

        if let Some(cursor_position) = self.cursor_position {
            
            if ui.input(|i| i.key_pressed(egui::Key::P)) {
                self.add_peak_marker(cursor_position.x);
                self.sort_peak_markers();
            }

            if ui.input(|i| i.key_pressed(egui::Key::B)) {
                self.add_background_marker(cursor_position.x);
                self.sort_background_markers();
            }

            if ui.input(|i| i.key_pressed(egui::Key::R)) {

                // there can only be 2 region markers
                if self.region_markers.len() > 1 {
                    self.clear_region_markers();
                }
                self.add_region_marker(cursor_position.x);
                self.sort_region_markers();
            }

         }

        ui.separator();

        ui.horizontal(|ui| {

            ui.label("Clear: ");

            if ui.button("Peaks").on_hover_text("Clear all peak markers").clicked() {
                self.clear_peak_markers();
            }

            if ui.button("Background").on_hover_text("Clear all background markers").clicked() {
                self.clear_background_markers();
            }

            if ui.button("Region").on_hover_text("Clear all region markers").clicked() {
                self.clear_region_markers();
            }

            ui.separator();

            if ui.button("Clear all").on_hover_text("Clear all markers").clicked() {
                self.clear_peak_markers();
                self.clear_background_markers();
                self.clear_region_markers();
            }
        });


    }

}

