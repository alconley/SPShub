// Conditional compilation
#[cfg(not(target_arch = "wasm32"))]
use crate::{
    plotter::app::PlotterApp, sps_cebra_eventbuilder::app::EVBApp as SPSCeBrAEvbApp,
    sps_eventbuilder::app::EVBApp as SPSEvbApp, sps_plot::app::SPSPlotApp,
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    // #[serde(skip)]
    #[cfg(not(target_arch = "wasm32"))]
    sps_cebra_evb_app: SPSCeBrAEvbApp,
    sps_cebra_evb_app_visible: bool,

    #[cfg(not(target_arch = "wasm32"))]
    sps_evb_app: SPSEvbApp,
    sps_evb_app_visible: bool,

    #[cfg(not(target_arch = "wasm32"))]
    sps_plot_app: SPSPlotApp,
    sps_plot_app_visible: bool,

    #[cfg(not(target_arch = "wasm32"))]
    plotter_app: PlotterApp,
    plotter_app_visible: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            sps_cebra_evb_app: SPSCeBrAEvbApp::default(), // Default initialization.
            sps_cebra_evb_app_visible: false,

            #[cfg(not(target_arch = "wasm32"))]
            sps_evb_app: SPSEvbApp::default(), // Default initialization.
            sps_evb_app_visible: false,

            #[cfg(not(target_arch = "wasm32"))]
            sps_plot_app: SPSPlotApp::default(),
            sps_plot_app_visible: false,

            #[cfg(not(target_arch = "wasm32"))]
            plotter_app: PlotterApp::default(),
            plotter_app_visible: false,
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
                sps_cebra_evb_app_visible: false,

                sps_evb_app: SPSEvbApp::new(cc), // Custom initialization.
                sps_evb_app_visible: false,

                sps_plot_app: SPSPlotApp::new(cc), // Custom initialization.
                sps_plot_app_visible: false,

                plotter_app: PlotterApp::new(cc), // Custom initialization.
                plotter_app_visible: false,
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

        egui::SidePanel::right("sidebar_panel").show(ctx, |ui| {
            // ui.heading("Apps");

            // Set the layout to top-down with centered alignment
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Apps");

                let full_width = ui.available_width(); // Get the full available width to make the label span the entire panel

                if ui
                    .add_sized(
                        [full_width, 0.0],
                        egui::SelectableLabel::new(self.sps_cebra_evb_app_visible, "SPS+CeBrA Evb"),
                    )
                    .clicked()
                {
                    self.sps_cebra_evb_app_visible = !self.sps_cebra_evb_app_visible;
                }
                if ui
                    .add_sized(
                        [full_width, 0.0],
                        egui::SelectableLabel::new(self.sps_evb_app_visible, "SPS Evb"),
                    )
                    .clicked()
                {
                    self.sps_evb_app_visible = !self.sps_evb_app_visible;
                }
                if ui
                    .add_sized(
                        [full_width, 0.0],
                        egui::SelectableLabel::new(self.sps_plot_app_visible, "SPS Plot"),
                    )
                    .clicked()
                {
                    self.sps_plot_app_visible = !self.sps_plot_app_visible;
                }

                if ui
                    .add_sized(
                        [full_width, 0.0],
                        egui::SelectableLabel::new(self.plotter_app_visible, "Plotter"),
                    )
                    .clicked()
                {
                    self.plotter_app_visible = !self.plotter_app_visible;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.heading("Tool Box");

            // Place utilities or warnings at the bottom, aligned to the left.
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                // Display a warning in debug builds about performance.
                egui::warn_if_debug_build(ui);
            });

            // conditional statement to differentiate between web and non-web targets.
            if cfg!(target_arch = "wasm32") {
                if self.sps_cebra_evb_app_visible {
                    // Instructions for web users, as event builders are not available.
                    egui::Window::new("SPS+CeBrA Eventbuilder").show(ctx, |ui| {
                        ui.label("Event builders are only available when compiling locally.");
                        ui.label("Download: 'git clone https://github.com/alconley/SPShub.git'");
                        ui.label("Run: 'cargo run --release'");
                    });
                }

                if self.sps_evb_app_visible {
                    // Instructions for web users, as event builders are not available.
                    egui::Window::new("SPS Eventbuilder").show(ctx, |ui| {
                        ui.label("Event builders are only available when compiling locally.");
                        ui.label("Download: 'git clone https://github.com/alconley/SPShub.git'");
                        ui.label("Run: 'cargo run --release'");
                    });
                }

                if self.sps_plot_app_visible {
                    egui::Window::new("SPS Plot").show(ctx, |ui| {
                        ui.label("Will be available soon. Hopefully lol");
                    });

                    // self.sps_plot_app.update(ctx, frame);
                }

                if self.plotter_app_visible {
                    egui::Window::new("Plotter").show(ctx, |ui| {
                        ui.label("Will be available soon. Hopefully lol");
                    });
                }
            } else {
                // Update calls for non-web targets are grouped to avoid repetitive conditional checks.
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if self.sps_cebra_evb_app_visible {
                        self.sps_cebra_evb_app.update(ctx, frame);
                    }

                    if self.sps_evb_app_visible {
                        self.sps_evb_app.update(ctx, frame);
                    }

                    if self.sps_plot_app_visible {
                        self.sps_plot_app.update(ctx, frame);
                    }

                    if self.plotter_app_visible {
                        self.plotter_app.update(ctx, frame);
                    }
                }
            }
        });
    }
}
