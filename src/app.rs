// Conditional compilation
#[cfg(not(target_arch = "wasm32"))]
use crate::{
    plotter::app::PlotterApp,
    sps_plot::app::SPSPlotApp,
};

use sps_eventbuilder::EVBApp as SPSEvbApp;
use cebra_sps_eventbuilder::EVBApp as SPSCeBrAEvbApp;
use cebra_eventbuilder::EVBApp as CeBrAEvbApp;

use crate::sps_runtime_estimator::app::SPSRunTimeApp;
use crate::cebra_efficiency::app::CeBrAEfficiencyApp;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
#[derive(Default)]
pub struct TemplateApp {
    sps_cebra_evb_app: SPSCeBrAEvbApp,
    sps_cebra_evb_app_visible: bool,

    sps_evb_app: SPSEvbApp,
    sps_evb_app_visible: bool,

    cebra_evb_app: CeBrAEvbApp,
    cebra_evb_app_visible: bool,

    #[cfg(not(target_arch = "wasm32"))]
    sps_plot_app: SPSPlotApp,
    sps_plot_app_visible: bool,

    #[cfg(not(target_arch = "wasm32"))]
    plotter_app: PlotterApp,
    plotter_app_visible: bool,

    sps_runtime_app: SPSRunTimeApp,
    sps_runtime_app_visible: bool,

    cebra_efficiency_app: CeBrAEfficiencyApp,
    cebra_efficiency_app_visible: bool,
}

impl TemplateApp {
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

                sps_plot_app: SPSPlotApp::new(cc),
                sps_plot_app_visible: false,

                plotter_app: PlotterApp::new(cc),
                plotter_app_visible: false,

                sps_runtime_app: SPSRunTimeApp::new(cc),
                sps_runtime_app_visible: false,

                cebra_efficiency_app: CeBrAEfficiencyApp::new(cc),
                cebra_efficiency_app_visible: false,
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

                    if ui
                        .add_sized(
                            [full_width, 0.0],
                            egui::SelectableLabel::new(self.cebra_efficiency_app_visible, "Efficiency"),
                        )
                        .clicked()
                    {
                        self.cebra_efficiency_app_visible = !self.cebra_efficiency_app_visible;
                    }

                    ui.separator();

                    ui.heading("General");

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
                    egui::Window::new("SPS Plot").show(ctx, |ui| {
                        ui.label("Will be available soon. Hopefully lol");
                    });

                }

                if self.plotter_app_visible {
                    egui::Window::new("Plotter").show(ctx, |ui| {
                        ui.label("Will be available soon. Hopefully lol");
                    });
                }

                if self.sps_runtime_app_visible {
                    self.sps_runtime_app.update(ctx, frame);
                }

                if self.cebra_efficiency_app_visible {
                    self.cebra_efficiency_app.update(ctx, frame);
                }
            } else {
                // Update calls for non-web targets are grouped to avoid repetitive conditional checks.
                #[cfg(not(target_arch = "wasm32"))]
                {
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

                    if self.plotter_app_visible {
                        self.plotter_app.update(ctx, frame);
                    }

                    if self.sps_runtime_app_visible {
                        self.sps_runtime_app.update(ctx, frame);
                    }

                    if self.cebra_efficiency_app_visible {
                        self.cebra_efficiency_app.update(ctx, frame);
                    }
                }
            }
        });
    }
}
