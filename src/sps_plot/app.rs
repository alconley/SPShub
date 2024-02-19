use log::info;

use eframe::egui::{self, RichText, Color32};
use eframe::App;
use egui_plot::{Plot, Points, PlotPoints, PlotPoint, Legend, MarkerShape, Text, VLine, PlotBounds};

use std::f64::consts::PI;
use std::collections::HashMap;

use super::nuclear_data::{MassMap, NuclearData};
use super::excitation_fetchor::ExcitationFetcher;

const C: f64 = 299792458.0; // Speed of light in m/s
const QBRHO2P: f64 = 1.0E-9 * C; // Converts qbrho to momentum (p) (kG*cm -> MeV/c)


#[derive(Clone, serde::Deserialize, serde::Serialize, Debug, Default)]
pub struct Reaction {
    pub target_z: i32,
    pub target_a: i32,
    pub target_data: Option<NuclearData>,

    pub projectile_z: i32,
    pub projectile_a: i32,
    pub projectile_data: Option<NuclearData>,

    pub ejectile_z: i32,
    pub ejectile_a: i32,
    pub ejectile_data: Option<NuclearData>,

    pub resid_z: i32,
    pub resid_a: i32,
    pub resid_data: Option<NuclearData>,

    pub reaction_identifier: String,

    pub excitation_levels: Vec<f64>,

    // Fields for the associated nuclear data
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct SPSPlotApp {
    sps_angle: f64,
    beam_energy: f64,
    magnetic_field: f64,
    rho_min: f64,
    rho_max: f64,
    reactions: Vec<Reaction>,
    reaction_data: HashMap<String, Vec<(f64, f64)>>,
    is_loading: bool,
}

impl SPSPlotApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {

        Self {
            sps_angle: 35.0, // degree
            beam_energy: 16.0, // MeV
            magnetic_field: 8.7, // kG
            rho_min: 69.0,
            rho_max: 87.0,
            // reactions: Reaction::default(),
            reactions: Vec::new(),
            reaction_data: HashMap::new(),
            is_loading: false,
        }
    }

    fn sps_settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("SE-SPS Settings").color(Color32::LIGHT_BLUE).size(18.0));

        ui.horizontal(|ui| {
            ui.label("SPS Angle: ").on_hover_text("SE-SPS's angle currently limited to 60°");
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

            // if ui.button("Calculate").clicked() {
            //     // self.calculate_rho_for_all();
            //     // self.calculate_rho_from_levels(&self.reactions.clone());
            //     self.calculate_rho_for_all();
            // }

            if ui.button("+").clicked() {
                self.reactions.push(Reaction::default());
            }

        });

        ui.separator();

        let mut index_to_remove: Option<usize> = None;

        for (index, reaction) in self.reactions.iter_mut().enumerate() {

            ui.horizontal(|ui| {

                // let reaction = &mut self.reactions; 
                ui.label(format!("Reaction {}", index));

                ui.separator();

                if ui.button("-").clicked() {
                    index_to_remove = Some(index);
                }

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

                ui.label(format!("{}", reaction.reaction_identifier));
                
                if ui.button("Get Reaction").clicked() {

                    Self::populate_reaction_data(reaction);
                    Self::fetch_excitation_levels(reaction);
                }

                ui.separator();

            });
        }

        if let Some(index) = index_to_remove {
            self.reactions.remove(index);
        }


    }

    fn populate_reaction_data(reaction: &mut Reaction) {
        let mass_map = match MassMap::new() {
            Ok(map) => map,
            Err(e) => {
                log::error!("Failed to initialize MassMap: {}", e);
                // Handle error appropriately, maybe by creating an empty MassMap or exiting
                MassMap::default() // Placeholder, adjust according to your error handling strategy
            }
        };

        reaction.target_data = mass_map.get_data(&(reaction.target_z as u32), &(reaction.target_a as u32)).cloned();
        reaction.projectile_data = mass_map.get_data(&(reaction.projectile_z as u32), &(reaction.projectile_a as u32)).cloned();
        reaction.ejectile_data = mass_map.get_data(&(reaction.ejectile_z as u32), &(reaction.ejectile_a as u32)).cloned();

        reaction.resid_z = (reaction.target_z + reaction.projectile_z - reaction.ejectile_z) as i32;
        reaction.resid_a = (reaction.target_a + reaction.projectile_a - reaction.ejectile_a) as i32;

        reaction.resid_data = mass_map.get_data(&(reaction.resid_z as u32), &(reaction.resid_a as u32)).cloned();
        
        reaction.reaction_identifier = format!(
            "{}({},{}){}",
            reaction.target_data.as_ref().map_or("None", |data| &data.isotope),
            reaction.projectile_data.as_ref().map_or("None", |data| &data.isotope),
            reaction.ejectile_data.as_ref().map_or("None", |data| &data.isotope),
            reaction.resid_data.as_ref().map_or("None", |data| &data.isotope)
        );

        info!("Reaction: {:?}", reaction);
    }
        
    fn fetch_excitation_levels(reaction: &mut Reaction) -> Vec<f64> {

        let isotope = reaction.resid_data.as_ref().map_or("None", |data| &data.isotope);
        if isotope == "None" {
            log::error!("No isotope found for reaction: {}", reaction.reaction_identifier);
            return vec![];
        }

        // Using an async block, note that this requires an executor to run the block synchronously
        let fetcher = ExcitationFetcher::new();
        fetcher.fetch_excitation_levels(&isotope);

        let levels_lock = fetcher.excitation_levels.lock().unwrap();
        let error_lock = fetcher.error_message.lock().unwrap();

        if let Some(levels) = &*levels_lock {
            info!("Fetched excitation levels: {:?}", levels);
            return levels.clone();
        } else if let Some(error) = &*error_lock {
            log::error!("Error fetching excitation levels: {}", error);
            return vec![];
        }

        vec![]
    }

    /*
    fn calculate_rho_from_levels(&mut self, reaction: &Reaction) {

        // clear the reaction data

        // Extract data needed for immutable borrows before mutable operations
        let target_data = self.mass_map.get_data(&(reaction.target_z as u32), &(reaction.target_a as u32)).cloned();
        let projectile_data = self.mass_map.get_data(&(reaction.projectile_z as u32), &(reaction.projectile_a as u32)).cloned();
        let ejectile_data = self.mass_map.get_data(&(reaction.ejectile_z as u32), &(reaction.ejectile_a as u32)).cloned();
        
        if let (Some(target), Some(projectile), Some(ejectile)) = (target_data, projectile_data, ejectile_data) {

            let resid_z = target.z + projectile.z - ejectile.z;
            let resid_a = target.a + projectile.a - ejectile.a;

            if let Some(resid) = self.mass_map.get_data(&resid_z, &resid_a).cloned() {
                let reaction_identifier = format!("{}({},{}){}", target.isotope, projectile.isotope, ejectile.isotope, resid.isotope);
                info!("Reaction: {}", reaction_identifier);

                let mut reaction_values = Vec::new();

                let q_value = target.mass + projectile.mass - ejectile.mass - resid.mass;

                for excitation in &reaction.excitation_levels {
                    let reaction_q_value = q_value - excitation;
                    let beam_reaction_energy = self.beam_energy; // could put energy loss through target here

                    let _threshold = - reaction_q_value * (ejectile.mass + resid.mass) / (ejectile.mass + resid.mass - projectile.mass);
                    let term1 = (projectile.mass * ejectile.mass * beam_reaction_energy).sqrt() / (ejectile.mass + resid.mass) * (self.sps_angle * PI / 180.0).cos();
                    let term2 = (beam_reaction_energy * (resid.mass - projectile.mass) + resid.mass * reaction_q_value) / (ejectile.mass + resid.mass);

                    let ke1 = term1 + (term1*term1 + term2).sqrt();
                    let ke2 = term1 - (term1*term1 + term2).sqrt(); // spspy: spsplot has these as the same? copilot thought the second should be subtracted

                    let ejectile_energy = if ke1 > 0.0 { ke1 * ke1 } else { ke2 * ke2 }; 
                    
                    // convert ejectile ke to rho
                    let p = (ejectile_energy * (ejectile_energy + 2.0 * ejectile.mass)).sqrt();
                    let qbrho = p/QBRHO2P;
                    let rho =  qbrho / (self.magnetic_field * ejectile.z as f64);
                    info!("Excitation: {}, rho: {}", excitation, rho);

                    reaction_values.push((*excitation, rho));
                }

                self.reaction_data.insert((*reaction_identifier).to_string(), reaction_values);

            }
        }
    }

    fn calculate_rho_for_all(&mut self) {
        // Collect indices or clone reactions to avoid borrowing issues
        let reactions = self.reactions.clone(); // Example with cloning, adjust based on your needs
    
        for reaction in reactions {
            self.calculate_rho_from_levels(&reaction);
        }
    }

    fn plot_reaction_data(&self, ui: &mut egui::Ui) {
        let plot = Plot::new("SPS Plot")
            .show_y(false)
            .data_aspect(0.5);
            // .legend(Legend::default());
    
        plot.show(ui, |plot_ui| {

            // plots the rho values 
            plot_ui.vline(VLine::new(self.rho_min).color(Color32::LIGHT_GREEN));
            plot_ui.vline(VLine::new(self.rho_max).color(Color32::LIGHT_GREEN));


            // extract the reaction name from reaction_data and plot the rho values as x and the y as the index (0) for now
            for (index, (reaction, data)) in self.reaction_data.iter().enumerate() {

                for (excitation, rho) in data.iter() {

                    let points = Points::new([*rho, index as f64])
                    .name(excitation.clone())
                    .shape(MarkerShape::Circle)
                    .filled(true)
                    .color(Color32::LIGHT_BLUE)
                    .radius(4.0);

                    plot_ui.points(points);

                }
            }
        });
    }

    */
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

            // need a button to plot the data
            // self.plot_reaction_data(ui);

        });

    }
}



