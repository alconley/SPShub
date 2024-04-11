use sps_eventbuilder::EVBApp as SPSEvbApp;
use cebra_sps_eventbuilder::EVBApp as SPSCeBrAEvbApp;
use cebra_eventbuilder::EVBApp as CeBrAEvbApp;
use sps_plot::SPSPlotApp;
use sps_runtime_estimator::SPSRunTimeApp;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
#[derive(Default)]
pub struct SPSHubApp {
    sps_cebra_evb_app: SPSCeBrAEvbApp,
    sps_cebra_evb_app_visible: bool,

    sps_evb_app: SPSEvbApp,
    sps_evb_app_visible: bool,

    cebra_evb_app: CeBrAEvbApp,
    cebra_evb_app_visible: bool,

    sps_plot_app: SPSPlotApp,
    sps_plot_app_visible: bool,
    
    sps_runtime_app: SPSRunTimeApp,
    sps_runtime_app_visible: bool,

    // plotter_app: PlotterApp,
    // plotter_app_visible: bool,

}

impl SPSHubApp {
    /// Called once before the first frame.
    /// Creates a new instance of the application, possibly restoring from previous state.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let app = Self {
                sps_cebra_evb_app: SPSCeBrAEvbApp::new(cc, true),
                sps_cebra_evb_app_visible: false,

                sps_evb_app: SPSEvbApp::new(cc, true),
                sps_evb_app_visible: false,

                cebra_evb_app: CeBrAEvbApp::new(cc, true),
                cebra_evb_app_visible: false,

                sps_plot_app: SPSPlotApp::new(cc, true),
                sps_plot_app_visible: false,

                sps_runtime_app: SPSRunTimeApp::new(cc, true),
                sps_runtime_app_visible: false,

                // plotter_app: PlotterApp::new(cc),
                // plotter_app_visible: false,
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

impl eframe::App for SPSHubApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
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

        egui::SidePanel::right("sidebar_panel")
            .resizable(false)
            .show(ctx, |ui| {
                // ui.heading("Apps");

                // Set the layout to top-down with centered alignment
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let full_width = ui.available_width(); // Get the full available width to make the label span the entire panel

                    /* Event Builders */
                    ui.heading("Event Builders");

                    if ui
                        .add_sized(
                            [full_width, 0.0],
                            egui::SelectableLabel::new(self.sps_evb_app_visible, "SE-SPS"),
                        )
                        .clicked()
                    {
                        self.sps_evb_app_visible = !self.sps_evb_app_visible;
                    }

                    if ui
                        .add_sized(
                            [full_width, 0.0],
                            egui::SelectableLabel::new(self.cebra_evb_app_visible, "CeBrA"),
                        )
                        .clicked()
                    {
                        self.cebra_evb_app_visible = !self.cebra_evb_app_visible;
                    }

                    if ui
                        .add_sized(
                            [full_width, 0.0],
                            egui::SelectableLabel::new(self.sps_cebra_evb_app_visible, "SE-SPS+CeBrA"),
                        )
                        .clicked()
                    {
                        self.sps_cebra_evb_app_visible = !self.sps_cebra_evb_app_visible;
                    }

                    /* Event Builders */

                    ui.separator();

                    ui.heading("SE-SPS Utilities");

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
                            egui::SelectableLabel::new(self.sps_runtime_app_visible, "Run Time Estimator"),
                        )
                        .clicked()
                    {
                        self.sps_runtime_app_visible = !self.sps_runtime_app_visible;
                    }

                    ui.separator();

                    ui.heading("CeBrA Utilities");

                    ui.separator();

                    ui.heading("General");

                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Place utilities or warnings at the bottom, aligned to the left.
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                // Display a warning in debug builds about performance.
                egui::warn_if_debug_build(ui);
            });

            if self.sps_evb_app_visible {
                self.sps_evb_app.update(ctx, frame);
            }

            if self.sps_cebra_evb_app_visible {
                self.sps_cebra_evb_app.update(ctx, frame);
            }

            if self.cebra_evb_app_visible {
                self.cebra_evb_app.update(ctx, frame);
            }

            if self.sps_plot_app_visible {
                self.sps_plot_app.update(ctx, frame);
            }

            if self.sps_runtime_app_visible {
                self.sps_runtime_app.update(ctx, frame);
            }
        });
    }
}
