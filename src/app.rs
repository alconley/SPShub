// Conditional compilation
#[cfg(not(target_arch = "wasm32"))]
use crate::{
    sps_cebra_eventbuilder::app::EVBApp as SPSCeBrAEvbApp,
    sps_eventbuilder::app::EVBApp as SPSEvbApp,
};

use crate::sps_plot::app::SPSPlotApp;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    // #[serde(skip)]
    #[cfg(not(target_arch = "wasm32"))]
    sps_cebra_evb_app: SPSCeBrAEvbApp,

    #[cfg(not(target_arch = "wasm32"))]
    sps_evb_app: SPSEvbApp,

    sps_plot_app: SPSPlotApp,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            sps_cebra_evb_app: SPSCeBrAEvbApp::default(), // Default initialization.

            #[cfg(not(target_arch = "wasm32"))]
            sps_evb_app: SPSEvbApp::default(), // Default initialization.

            sps_plot_app: SPSPlotApp::default(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    /// Creates a new instance of the application, possibly restoring from previous state.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let app = Self {
                sps_cebra_evb_app: SPSCeBrAEvbApp::new(cc), // Custom initialization.
                sps_evb_app: SPSEvbApp::new(cc), // Custom initialization.
                sps_plot_app: SPSPlotApp::new(cc), // Custom initialization.
            };

            // Attempt to restore the app state from persistent storage, if available.
            if let Some(storage) = cc.storage {
                if let Some(state) = eframe::get_value::<Self>(storage, eframe::APP_KEY) {
                    return state; // Return the restored state.
                }
            }

            app // Return the newly initialized app.
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self::default() // WASM targets use default initialization.
        }
    }

}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Conley's Tool Box");

            // Place utilities or warnings at the bottom, aligned to the left.
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                // Display a warning in debug builds about performance.
                egui::warn_if_debug_build(ui);
            });

            // Use a clearer conditional statement to differentiate between web and non-web targets.
            if cfg!(target_arch = "wasm32") {
                // Instructions for web users, as event builders are not available.
                egui::Window::new("Event Builders").show(ctx, |ui| {
                    ui.label("Event builders are only available when compiling locally.");
                    ui.label("Download: 'git clone https://github.com/alconley/sps_cebra.git'");
                    ui.label("Run: 'cargo run --release'");
                });
                
                self.sps_plot_app.update(ctx, frame);

            } else {
                // Update calls for non-web targets are grouped to avoid repetitive conditional checks.
                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.sps_cebra_evb_app.update(ctx, frame);
                    self.sps_evb_app.update(ctx, frame);
                    self.sps_plot_app.update(ctx, frame);

                }
            }
        });

    }
}
