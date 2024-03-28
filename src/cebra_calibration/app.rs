use eframe::egui::{self};
use eframe::App;

use super::efficiency::GammaSource;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct CeBrAEfficiencyApp {
    gamma_sources: Vec<GammaSource>,
}

impl CeBrAEfficiencyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            gamma_sources: vec![],
        }
    }

    fn remove_gamma_source(&mut self, index: usize) {
        self.gamma_sources.remove(index);
    }
}

impl App for CeBrAEfficiencyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("CeBrA Efficiency").show(ctx, |ui| {
            if ui.button("Add Gamma Source").clicked() {
                self.gamma_sources.push(GammaSource::new());
            }

            ui.separator();

            ui.label("Gamma Sources");
            
            let mut index_to_remove: Option<usize> = None;

            // Iterate over gamma_sources with enumeration to provide unique identifiers
            for (index, gamma_source) in self.gamma_sources.iter_mut().enumerate() {

                gamma_source.source_ui(ui);

                if ui.button("Remove").clicked() {
                    index_to_remove = Some(index);
                }

                ui.separator();

            }

            if let Some(index) = index_to_remove {
                self.remove_gamma_source(index);
            }

        });
    }
}
