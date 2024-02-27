use egui_plot::{Points, PlotPoints, PlotUi};
use egui_plot::Polygon as EguiPolygon;

use eframe::egui::{Color32, Stroke};

use std::fs::File;
use std::io::{BufReader, Write};

use serde::{Serialize, Deserialize};
use serde_json;

use rfd::FileDialog;

use geo::{Point, Polygon, LineString, algorithm::contains::Contains};

// typical cut names for sps experiments
const CUT_COLUMN_NAMES: &[&str] = &[
    "AnodeBackEnergy", "AnodeFrontEnergy", "Cathode",
     "ScintLeftEnergy", "Xavg", "X1", "X2"
];

#[derive(Serialize, Deserialize, Default)]
pub struct EditableEguiPolygon {
    pub vertices: Vec<[f64; 2]>,        // List of vertex coordinates
    selected_vertex_index: Option<usize>,  // Index of the selected vertex (if any)
    pub selected_x_column: Option<String>,
    pub selected_y_column: Option<String>,
}

impl EditableEguiPolygon {
    /// Creates a new `EditablePolygon` with default vertices.
    /// Current Cut Binds: 
    ///     Right click to add verticies 
    ///     Left click to remove verticies
    ///     Middle click to remove all verticies
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),  // Initialize with an empty set of vertices
            selected_vertex_index: None,  // Initially, no vertex is selected
            selected_x_column: None,
            selected_y_column: None,
        }
    }

    pub fn draw(&mut self, plot_ui: &mut PlotUi) {
        self.handle_mouse_interactions(plot_ui);   // Handle mouse interactions
        self.draw_vertices_and_polygon(plot_ui);   // Draw vertices and polygon
    }


    fn handle_mouse_interactions(&mut self, plot_ui: &mut PlotUi) {
        let response = plot_ui.response();

        if response.clicked() {
            let pointer_pos = plot_ui.pointer_coordinate().unwrap();
            self.add_new_vertex([pointer_pos.x, pointer_pos.y]); // Add a new vertex on left-click
        }

        if response.secondary_clicked() {
            let pointer_pos = plot_ui.pointer_coordinate().unwrap();
            self.selected_vertex_index = self.get_closest_vertex_index([pointer_pos.x, pointer_pos.y]); // Select and remove on right-click
            self.remove_vertex();
        }

        if response.middle_clicked() {
            self.remove_all_vertices(); // Remove all vertices on middle-click
        }
    }

    fn add_new_vertex(&mut self, coordinates: [f64; 2]) {
        self.vertices.push(coordinates); // Add a new vertex to the list
    }

    fn remove_vertex(&mut self) {
        if let Some(index) = self.selected_vertex_index {
            self.vertices.remove(index); // Remove the selected vertex
            self.clear_selection(); // Clear the selection
        }
    }

    fn get_closest_vertex_index(&self, pointer_pos: [f64; 2]) -> Option<usize> {
        let mut closest_vertex_index: Option<usize> = None;
        let mut closest_distance: f64 = 0.0;

        for (index, vertex) in self.vertices.iter().enumerate() {
            let distance = (vertex[0] - pointer_pos[0]).powi(2) + (vertex[1] - pointer_pos[1]).powi(2);
            if closest_vertex_index.is_none() || distance < closest_distance {
                closest_vertex_index = Some(index);
                closest_distance = distance;
            }
        }

        closest_vertex_index
    }

    fn clear_selection(&mut self) {
        self.selected_vertex_index = None; // Clear the selected vertex
    }

    fn remove_all_vertices(&mut self) {
        self.vertices.clear(); // Remove all vertices
        self.clear_selection(); // Clear the selection
    }

    fn draw_vertices_and_polygon(&mut self, plot_ui: &mut PlotUi) {
        if !self.vertices.is_empty() {
            let color = Color32::RED;
            let plot_points = PlotPoints::new(self.vertices.clone());
            let polygon_points = EguiPolygon::new(plot_points).fill_color(Color32::TRANSPARENT).stroke(Stroke::new(4.0, color));
            plot_ui.polygon(polygon_points); // Draw the polygon

            let vertices = Points::new(self.vertices.clone()).radius(5.0).color(color);
            plot_ui.points(vertices); // Draw the vertices
        }
    }

    pub fn save_cut_to_json(&self) -> Result<(), Box<dyn std::error::Error>> {

        // Create a default file name based on the selected columns
        let default_name = match (&self.selected_x_column, &self.selected_y_column) {
                (Some(x), Some(y)) => format!("{}_{}_cut.json", y, x),
                _ => "cut.json".to_string(),
            };
    
        if let Some(file_path) = FileDialog::new()
            .set_file_name(default_name) 
            .add_filter("JSON Files", &["json"])  // Add a filter for json files
            .save_file() {
    
                let serialized = serde_json::to_string(self)?;
                let mut file = File::create(file_path)?;
                file.write_all(serialized.as_bytes())?;
        }
        Ok(())
    }

    pub fn load_cut_from_json(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        if let Some(file_path) = FileDialog::new()
            .set_file_name("cut.json")  // Suggest a default file name for convenience
            .add_filter("JSON Files", &["json"])  // Filter for json files
            .pick_file() {

                let file = File::open(file_path)?;
                let reader = BufReader::new(file);
                let loaded_polygon: EditableEguiPolygon = serde_json::from_reader(reader)?;
                *self = loaded_polygon;
        }
        Ok(())
    }

    fn to_geo_polygon(&self) -> Polygon<f64> {
        let exterior_coords: Vec<_> = self.vertices.iter()
            .map(|&[x, y]| (x, y))
            .collect();
        let exterior_line_string = LineString::from(exterior_coords);
        Polygon::new(exterior_line_string, vec![])
    }

    pub fn is_inside(&self, x: f64, y: f64) -> bool {
        let point = Point::new(x, y);
        let polygon = self.to_geo_polygon();
        polygon.contains(&point)
    }

    pub fn cut_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {

            ui.separator();

            // Y Column ComboBox
            egui::ComboBox::from_label("Y Column")
            .selected_text(self.selected_y_column.as_deref().unwrap_or(""))
            .show_ui(ui, |ui| {
                for &column in CUT_COLUMN_NAMES.iter() {
                    if ui.selectable_label(self.selected_y_column.as_deref() == Some(column), column).clicked() {
                        self.selected_y_column = Some(column.to_string());
                    }
                }
            });

            ui.separator();

            // X Column ComboBox
            egui::ComboBox::from_label("X Column")
                .selected_text(self.selected_x_column.as_deref().unwrap_or(""))
                .show_ui(ui, |ui| {
                    for &column in CUT_COLUMN_NAMES.iter() {
                        if ui.selectable_label(self.selected_x_column.as_deref() == Some(column), column).clicked() {
                            self.selected_x_column = Some(column.to_string());
                        }
                    }
                });

            ui.separator();

            // Load Cut button
            if ui.button("Load Cut").clicked() {
                if let Err(e) = self.load_cut_from_json() {
                    eprintln!("Error loading cut: {:?}", e);
                }
            }

            // Save Cut button
            let can_save: bool = self.selected_x_column.is_some() && self.selected_y_column.is_some();
            if ui.add_enabled(can_save, egui::Button::new("Save Cut")).clicked() {
                if let Err(e) = self.save_cut_to_json() {
                    eprintln!("Error saving cut: {:?}", e);
                }
            }

            ui.separator();

        });
    }

}




