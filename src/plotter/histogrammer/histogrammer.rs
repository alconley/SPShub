use eframe::egui::{Color32, Stroke};
use std::collections::HashMap;

use rfd::FileDialog;
use serde_json; // or use serde_yaml for YAML serialization
use std::fs::File;

use egui_plot::{Bar, BarChart, Line, Orientation, PlotPoints};
use polars::prelude::*;

use super::histogram1d::Histogram;
use super::histogram2d::Histogram2D;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum HistogramTypes {
    Hist1D(Histogram),
    Hist2D(Histogram2D),
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Histogrammer {
    pub histogram_list: HashMap<String, HistogramTypes>,
}

impl Histogrammer {
    // Creates a new instance of Histogrammer.
    pub fn new() -> Self {
        Self {
            histogram_list: HashMap::new(),
        }
    }

    // Adds a new 1D histogram to the histogram list.
    pub fn add_hist1d(&mut self, name: &str, bins: usize, range: (f64, f64)) {
        let hist: Histogram = Histogram::new(bins, range); // Create a new histogram.
        self.histogram_list
            .insert(name.to_string(), HistogramTypes::Hist1D(hist)); // Store it in the hashmap.
    }

    // Fills a 1D histogram with data from a polars dataframe/column.
    pub fn fill_hist1d(&mut self, name: &str, lf: &LazyFrame, column_name: &str) -> bool {
        let hist: &mut Histogram = match self.histogram_list.get_mut(name) {
            Some(HistogramTypes::Hist1D(hist)) => hist,
            _ => return false, // Return false if the histogram doesn't exist.
        };

        // Attempt to collect the LazyFrame into a DataFrame
        let df_result = lf
            .clone()
            .filter(col(column_name).neq(lit(-1e6))) // Filter out the -1e6 values.
            .select([col(column_name)])
            .collect();

        // Handle the Result before proceeding
        match df_result {
            Ok(df) => {
                // Now that we have a DataFrame, we can attempt to convert it to an ndarray
                let ndarray_df_result = df.to_ndarray::<Float64Type>(IndexOrder::Fortran);

                match ndarray_df_result {
                    Ok(ndarray_df) => {
                        // You now have the ndarray and can proceed with your logic
                        let shape = ndarray_df.shape();
                        let rows = shape[0];

                        // Iterating through the ndarray and filling the histogram
                        for i in 0..rows {
                            let value = ndarray_df[[i, 0]];
                            hist.fill(value);
                        }

                        true
                    }
                    Err(e) => {
                        // Handle the error, for example, log it or return an error
                        eprintln!("Failed to convert DataFrame to ndarray: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                // Handle the error, for example, log it or return an error
                eprintln!("Failed to collect LazyFrame: {}", e);
                false
            }
        }
    }

    // Adds and fills a 1D histogram with data from a Polars LazyFrame.
    pub fn add_fill_hist1d(
        &mut self,
        name: &str,
        lf: &LazyFrame,
        column_name: &str,
        bins: usize,
        range: (f64, f64),
    ) {
        self.add_hist1d(name, bins, range); // Add the histogram.
        self.fill_hist1d(name, lf, column_name); // Fill it with data.
    }

    // Generates a histogram using the bar chart from the `egui` library.
    pub fn egui_histogram_step(&self, name: &str, color: Color32) -> Option<Line> {
        if let Some(HistogramTypes::Hist1D(hist)) = self.histogram_list.get(name) {
            let line_points = hist.step_histogram_points();

            // Convert line_points to a Vec<[f64; 2]>
            let plot_points: PlotPoints = line_points.iter().map(|&(x, y)| [x, y]).collect();

            Some(Line::new(plot_points).color(color).name(name))
        } else {
            None
        }
    }

    // Adds a new 2D histogram to the histogram list.
    pub fn add_hist2d(
        &mut self,
        name: &str,
        x_bins: usize,
        x_range: (f64, f64),
        y_bins: usize,
        y_range: (f64, f64),
    ) {
        let hist: Histogram2D = Histogram2D::new(x_bins, x_range, y_bins, y_range); // Create a new 2D histogram.
        self.histogram_list
            .insert(name.to_string(), HistogramTypes::Hist2D(hist)); // Store it in the hashmap.
    }

    // Fills a 2D histogram with x and y data.
    pub fn fill_hist2d(
        &mut self,
        name: &str,
        lf: &LazyFrame,
        x_column_name: &str,
        y_column_name: &str,
    ) -> bool {
        let hist: &mut Histogram2D = match self.histogram_list.get_mut(name) {
            Some(HistogramTypes::Hist2D(hist)) => hist,
            _ => return false, // Return false if the histogram doesn't exist.
        };

        // Attempt to collect the LazyFrame into a DataFrame
        let df_result = lf
            .clone()
            .select([col(x_column_name), col(y_column_name)])
            .filter(col(x_column_name).neq(lit(-1e6)))
            .filter(col(y_column_name).neq(lit(-1e6)))
            .collect();

        // Handle the Result before proceeding
        match df_result {
            Ok(df) => {
                // Now that we have a DataFrame, we can attempt to convert it to an ndarray
                let ndarray_df_result = df.to_ndarray::<Float64Type>(IndexOrder::Fortran);

                match ndarray_df_result {
                    Ok(ndarray_df) => {
                        // You now have the ndarray and can proceed with your logic
                        let shape = ndarray_df.shape();
                        let rows = shape[0];

                        // Iterating through the ndarray rows and filling the 2D histogram
                        for i in 0..rows {
                            let x_value = ndarray_df[[i, 0]];
                            let y_value = ndarray_df[[i, 1]];

                            hist.fill(x_value, y_value);
                        }

                        true
                    }
                    Err(e) => {
                        // Handle the error, for example, log it or return an error
                        eprintln!("Failed to convert DataFrame to ndarray: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                // Handle the error, for example, log it or return an error
                eprintln!("Failed to collect LazyFrame: {}", e);
                false
            }
        }
    }

    // Adds and fills a 2D histogram with data from Polars LazyFrame columns.
    pub fn add_fill_hist2d(
        &mut self,
        name: &str,
        lf: &LazyFrame,
        x_column_name: &str,
        x_bins: usize,
        x_range: (f64, f64),
        y_column_name: &str,
        y_bins: usize,
        y_range: (f64, f64),
    ) {
        self.add_hist2d(name, x_bins, x_range, y_bins, y_range); // Add the histogram.
        self.fill_hist2d(name, lf, x_column_name, y_column_name); // Fill it with data.
    }

    // Generates a heatmap using the `egui` library based on a 2D histogram.
    pub fn egui_heatmap(&self, name: &str) -> Option<BarChart> {
        if let Some(HistogramTypes::Hist2D(hist)) = self.histogram_list.get(name) {
            let bars_data = hist.generate_bar_data();
            let mut bars = Vec::new();

            let min: u32 = hist.min_count;
            let max: u32 = hist.max_count;
            for bar_data in bars_data {
                let color: Color32 = viridis_colormap(bar_data.count, min, max); // Determine color based on the count, using a colormap.

                let bar = Bar {
                    orientation: Orientation::Vertical,
                    argument: bar_data.x,
                    value: bar_data.height,
                    bar_width: bar_data.bar_width,
                    fill: color,
                    stroke: Stroke::new(1.0, color),
                    name: format!("x = {}\ny = {}\n{}", bar_data.x, bar_data.y, bar_data.count),
                    base_offset: Some(bar_data.y - bar_data.height / 2.0),
                };
                bars.push(bar);
            }

            // Return a BarChart object if the histogram exists, otherwise return None.
            Some(BarChart::new(bars).name(name))
        } else {
            None
        }
    }

    // additional functions
    pub fn get_histogram_list(&self) -> Vec<String> {
        let mut names: Vec<String> = self.histogram_list.keys().cloned().collect();
        names.sort();

        names
    }

    pub fn get_histogram_type(&self, name: &str) -> Option<&HistogramTypes> {
        self.histogram_list.get(name)
    }

    // Function to save the Histogrammer as JSON using a file dialog
    pub fn _save_to_json_with_dialog(&self) -> Result<(), std::io::Error> {
        if let Some(path) = FileDialog::new()
            .set_title("Save as JSON")
            .add_filter("JSON files", &["json"])
            .save_file() {
            let file = File::create(path)?;
            let writer = std::io::BufWriter::new(file);
            serde_json::to_writer_pretty(writer, &self)?;
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "No file selected"))
        }
    }

}

// Function to generate a color based on a value using the Viridis colormap, the matplotlib default.
fn viridis_colormap(value: u32, min: u32, max: u32) -> Color32 {
    // Handle case where min == max to avoid division by zero
    let normalized: f64 = if max > min {
        (value as f64 - min as f64) / (max as f64 - min as f64)
    } else {
        0.0
    }
    .clamp(0.0, 1.0);

    // Key colors from the Viridis colormap
    let viridis_colors: [(f32, f32, f32); 32] = [
        (0.267003985, 0.004872566, 0.329415069),
        (0.277228998, 0.051716984, 0.37694991),
        (0.28247969, 0.097334964, 0.419510575),
        (0.282711276, 0.139317688, 0.456197068),
        (0.278092635, 0.179895883, 0.486377421),
        (0.269137787, 0.219429659, 0.50989087),
        (0.256733532, 0.257754383, 0.52718378),
        (0.242031461, 0.294643816, 0.539209024),
        (0.226243756, 0.329989329, 0.547162826),
        (0.210443168, 0.363856061, 0.552221276),
        (0.195412486, 0.396435844, 0.555350926),
        (0.181477325, 0.428017314, 0.557198854),
        (0.168574228, 0.458905237, 0.55806733),
        (0.156365949, 0.489384598, 0.557941172),
        (0.144535294, 0.519685615, 0.556527663),
        (0.133249552, 0.549958247, 0.553339219),
        (0.123833067, 0.580259243, 0.547771637),
        (0.119442112, 0.610546221, 0.53918201),
        (0.124881902, 0.640695014, 0.526954942),
        (0.144277738, 0.670499732, 0.510554716),
        (0.178281445, 0.699705646, 0.489567134),
        (0.224797439, 0.72801441, 0.463677887),
        (0.281243458, 0.755097766, 0.432683204),
        (0.345693489, 0.780604757, 0.396465689),
        (0.416705432, 0.80418531, 0.355029985),
        (0.493228829, 0.825506231, 0.308497657),
        (0.574270238, 0.844288831, 0.257257704),
        (0.658654029, 0.860389968, 0.202434461),
        (0.744780537, 0.873933018, 0.147547821),
        (0.830610047, 0.885437755, 0.10427358),
        (0.91400241, 0.895811264, 0.100134278),
        (0.993248149, 0.906154763, 0.143935944),
    ];

    // Interpolate between colors in the colormap
    let scaled_val: f64 = normalized * (viridis_colors.len() - 1) as f64;
    let index: usize = scaled_val.floor() as usize;
    let fraction: f32 = scaled_val.fract() as f32;

    let color1: (f32, f32, f32) = viridis_colors[index];
    let color2: (f32, f32, f32) = viridis_colors[(index + 1).min(viridis_colors.len() - 1)];

    let red: f32 = (color1.0 + fraction * (color2.0 - color1.0)) * 255.0;
    let green: f32 = (color1.1 + fraction * (color2.1 - color1.1)) * 255.0;
    let blue: f32 = (color1.2 + fraction * (color2.2 - color1.2)) * 255.0;

    Color32::from_rgb(red as u8, green as u8, blue as u8)
}
