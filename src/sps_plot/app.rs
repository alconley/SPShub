use log::info;

use eframe::egui::{self, RichText, Color32};
use eframe::App;
use egui_plot::{Plot, Points, PlotPoints, Legend, MarkerShape};

use reqwest;
use scraper::{Html, Selector};
use regex::Regex;

use std::f64::consts::PI;
use std::error::Error;
use std::collections::HashMap;

use super::nuclear_data::MassMap;

const C: f64 = 299792458.0; // Speed of light in m/s
const QBRHO2P: f64 = 1.0E-9 * C; // Converts qbrho to momentum (p) (kG*cm -> MeV/c)


#[derive(Clone, serde::Deserialize, serde::Serialize, Debug, Default)]
pub struct Reaction {
    target_z: i32,
    target_a: i32,
    projectile_z: i32,
    projectile_a: i32,
    ejectile_z: i32,
    ejectile_a: i32,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct SPSPlotApp {
    sps_angle: f64,
    beam_energy: f64,
    magnetic_field: f64,
    rho_min: f64,
    rho_max: f64,
    reactions: Vec<Reaction>,
    mass_map: MassMap,
    reaction_data: HashMap<String, Vec<(f64, f64)>>,
}

impl SPSPlotApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {

        let mass_map = match MassMap::new() {
            Ok(map) => map,
            Err(e) => {
                log::error!("Failed to initialize MassMap: {}", e);
                // Handle error appropriately, maybe by creating an empty MassMap or exiting
                MassMap::default() // Placeholder, adjust according to your error handling strategy
            }
        };

        Self {
            sps_angle: 35.0, // degree
            beam_energy: 16.0, // MeV
            magnetic_field: 8.7, // kG
            rho_min: 69.0,
            rho_max: 87.0,
            mass_map,
            reactions: Vec::new(),
            reaction_data: HashMap::new(),
        }
    }

    fn sps_settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("SE-SPS Settings").color(Color32::LIGHT_BLUE).size(18.0));

        ui.horizontal(|ui| {
            ui.label("SPS Angle: ");
            ui.add(egui::DragValue::new(&mut self.sps_angle)
                .suffix("°")
                .clamp_range(0.0..=60.0));

            ui.label("Beam Energy: ");
            ui.add(egui::DragValue::new(&mut self.beam_energy)
                .suffix(" MeV")
                .clamp_range(0.0..=f64::MAX));

            ui.label("Magnetic Field: ");
            ui.add(egui::DragValue::new(&mut self.magnetic_field)
                .suffix(" kG")
                .clamp_range(0.0..=17.0));

            ui.label("Rho Min: ").on_hover_text("SE-SPS Rho Min is usually 69.0");
            ui.add(egui::DragValue::new(&mut self.rho_min)
                .suffix(" cm")
                .clamp_range(0.0..=f64::MAX));

            ui.label("Rho Max: ").on_hover_text("SE-SPS Rho Max is usually 87.0");
            ui.add(egui::DragValue::new(&mut self.rho_max)
                .suffix(" cm")
                .clamp_range(0.0..=f64::MAX));
        });
    }

    fn reaction_ui(&mut self, ui: &mut egui::Ui) {

        ui.horizontal(|ui| {

            ui.label(RichText::new("Reactions").color(Color32::LIGHT_BLUE).size(18.0));

            ui.separator();

            if ui.button("Add Reaction").clicked() {
                self.reactions.push(Reaction::default());
            }

            ui.separator();

            if ui.button("Get Nuclear Data for All Reactions").clicked() {
                self.get_nuclear_data_for_all_reactions();
            }

        });

        let mut to_remove = Vec::new();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (index, reaction) in self.reactions.iter_mut().enumerate() {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!("Reaction {}", index));
                    ui.separator();
                    ui.label("Target: ");
                    ui.add(egui::DragValue::new(&mut reaction.target_z).prefix("Z: "));
                    ui.add(egui::DragValue::new(&mut reaction.target_a).prefix("A: "));
                    ui.separator();

                    ui.label("Projectile: ");
                    ui.add(egui::DragValue::new(&mut reaction.projectile_z).prefix("Z: "));
                    ui.add(egui::DragValue::new(&mut reaction.projectile_a).prefix("A: "));
                    ui.separator();

                    ui.label("Ejectile: ");
                    ui.add(egui::DragValue::new(&mut reaction.ejectile_z).prefix("Z: "));
                    ui.add(egui::DragValue::new(&mut reaction.ejectile_a).prefix("A: "));
                    ui.separator();
                
                    if ui.button("Remove").clicked() {
                        to_remove.push(index);
                    }

                });
            };
        });

        // Remove reactions in reverse order to avoid shifting indices
        for index in to_remove.into_iter().rev() {
            self.reactions.remove(index);
        }

    }

    fn get_nuclear_data_for_reaction(&mut self, reaction: &Reaction) {
        // Extract data needed for immutable borrows before mutable operations
        let target_data = self.mass_map.get_data(&(reaction.target_z as u32), &(reaction.target_a as u32)).cloned();
        let projectile_data = self.mass_map.get_data(&(reaction.projectile_z as u32), &(reaction.projectile_a as u32)).cloned();
        let ejectile_data = self.mass_map.get_data(&(reaction.ejectile_z as u32), &(reaction.ejectile_a as u32)).cloned();
        
        if let (Some(target), Some(projectile), Some(ejectile)) = (target_data, projectile_data, ejectile_data) {
            // You can now use `target`, `projectile`, and `ejectile` here
            // Immutable borrow of `self.mass_map` is no longer in scope

            let resid_z = target.z + projectile.z - ejectile.z;
            let resid_a = target.a + projectile.a - ejectile.a;

            // This is now acceptable since previous immutable borrows are out of scope
            if let Some(resid) = self.mass_map.get_data(&resid_z, &resid_a).cloned() {
                let isotope = resid.isotope.clone(); // Clone isotope for later use
                let reaction_identifier = format!("{}({},{}){}", target.isotope, projectile.isotope, ejectile.isotope, resid.isotope);
                info!("Reaction: {}", reaction_identifier);

                let mut reaction_values = Vec::new();

                let q_value = target.mass + projectile.mass - ejectile.mass - resid.mass;

                // Continue with your logic...
                // You can now safely call `self.get_excitations` since `self.mass_map.get_data` is not in scope
                let levels = self.get_excitations(&isotope).unwrap_or_default();
                info!("Levels: {:?}", levels);

                for excitation in levels {
                    let reaction_q_value = q_value - excitation;
                    let beam_reaction_energy = self.beam_energy; // could put energy loss through target here

                    let threshold = - reaction_q_value * (ejectile.mass + resid.mass) / (ejectile.mass + resid.mass - projectile.mass);
                    if beam_reaction_energy > threshold {
                        info!("Reaction is possible");
                    } else {
                        info!("Reaction is not possible");
                    }

                    let term1 = (projectile.mass * ejectile.mass * beam_reaction_energy).sqrt() / (ejectile.mass + resid.mass) * (self.sps_angle * PI / 180.0).cos();
                    let term2 = (beam_reaction_energy * (resid.mass - projectile.mass) + resid.mass * reaction_q_value) / (ejectile.mass + resid.mass);

                    let ke1 = term1 + (term1*term1 + term2).sqrt();
                    let ke2 = term1 - (term1*term1 + term2).sqrt(); // sps plot has these as the same? copilot thought the second should be subtracted

                    let ejectile_energy = if ke1 > 0.0 { ke1 * ke1 } else { ke2 * ke2 }; 
                    
                    // convert ejectile ke to rho
                    let p = (ejectile_energy * (ejectile_energy + 2.0 * ejectile.mass)).sqrt();
                    let qbrho = p/QBRHO2P;
                    let rho =  qbrho / (self.magnetic_field * ejectile.z as f64);
                    info!("Excitation: {}, rho: {}", excitation, rho);

                    reaction_values.push((excitation, rho));
                }

                self.reaction_data.insert(reaction_identifier, reaction_values);
            }
        }
    }

    fn get_nuclear_data_for_all_reactions(&mut self) {
        let reactions = self.reactions.clone(); // Consider the implications of cloning
        for reaction in reactions {
            self.get_nuclear_data_for_reaction(&reaction);
        }
    }

    fn get_excitations(&self, isotope: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        // Fetch the webpage content
        let url = format!("https://www.nndc.bnl.gov/nudat3/getdatasetClassic.jsp?nucleus={}&unc=nds", isotope);
        let site_content = reqwest::blocking::get(url)?.text()?;

        // Parse the HTML document
        let document = Html::parse_document(&site_content);
        let table_selector = Selector::parse("table").unwrap();

        // Attempt to select the specific table
        let tables = document.select(&table_selector).collect::<Vec<_>>();
        if tables.len() < 3 {
            return Err("Table not found or doesn't contain enough data".into());
        }

        // Prepare regex for cleaning and extracting numerical values
        let re_clean = Regex::new(r"\s*(\d+(\.\d+)?(E[+\-]?\d+)?)").unwrap();

        // Initialize a vector to hold the energy levels
        let mut levels = Vec::new();

        // Iterate over table rows, skipping the first header row
        for row in tables[2].select(&Selector::parse("tr").unwrap()).skip(1) {
            let entries = row.select(&Selector::parse("td").unwrap()).collect::<Vec<_>>();
            if !entries.is_empty() {
                let entry = &entries[0];
                let text = entry.text().collect::<Vec<_>>().join("");
                if let Some(caps) = re_clean.captures(&text) {
                    if let Some(matched) = caps.get(1) {
                        let cleaned_text = matched.as_str();
                        match cleaned_text.parse::<f64>() {
                            Ok(num) => levels.push(num / 1000.0), // Convert to MeV
                            Err(_) => continue, // Skip entries that can't be parsed as f64
                        }
                    }
                }
            }
        }

        Ok(levels)
    }

    pub fn plot_reaction_data(&self, ui: &mut egui::Ui) {
        let plot = Plot::new("SPS Plot")
            .legend(Legend::default());

        plot.show(ui, |plot_ui| {
            // Enumerate over reaction_data to get both the index and the data (rho values)
            for (index, (reaction, data)) in self.reaction_data.iter().enumerate() {
                // Map rho to x-axis and use the index for y-axis to differentiate reactions
                let points: Vec<_> = data.iter().map(|&(_excitation, rho)| [rho, index as f64]).collect();

                // Create PlotPoints from the Vec<[f64; 2]>
                let plot_points = PlotPoints::new(points);

                // Create and configure a Points object for the current reaction
                let points = Points::new(plot_points)
                    .name(reaction.clone()) // Set a name for legend
                    .shape(MarkerShape::Circle) // Use circle markers
                    .filled(true) // Markers will be filled
                    .color(Color32::RED) // Use red color for markers
                    .radius(4.0); // Set radius for markers

                // Add the configured Points object to the plot
                plot_ui.points(points);
            }
        });
    }
}

impl App for SPSPlotApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("SPS Plot").show(ctx, |ui| {

            egui::TopBottomPanel::top("top_panel").show_inside(ui, |ui| {
                self.sps_settings_ui(ui);
            });

            egui::TopBottomPanel::bottom("bottom_panel").show_inside(ui, |ui| {
                self.reaction_ui(ui);                
            
            });

                // Plot goes here
            self.plot_reaction_data(ui);

        });

    }
}

