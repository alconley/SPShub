
use egui_plot::{Line, PlotUi, PlotPoint, PlotPoints};
use egui::{Color32, Stroke};

use super::egui_markers::EguiFitMarkers;
use super::histogram1d::Histogram;

use nalgebra::DVector;
use varpro::prelude::*;
use varpro::solvers::levmar::{LevMarProblemBuilder, LevMarSolver};


fn gaussian(x: &DVector<f64>, mean: f64, std_dev: f64) -> DVector<f64> {
    x.map(|x_val| (-((x_val - mean).powi(2)) / (2.0 * std_dev.powi(2))).exp())
}

fn gaussian_pd_mean(x: &DVector<f64>, mean: f64, std_dev: f64) -> DVector<f64> {
    x.map(|x_val| (x_val - mean) / std_dev.powi(2) * (-((x_val - mean).powi(2)) / (2.0 * std_dev.powi(2))).exp())
}

fn gaussian_pd_std_dev(x: &DVector<f64>, mean: f64, std_dev: f64) -> DVector<f64> {
    x.map(|x_val| {
        let exponent = -((x_val - mean).powi(2)) / (2.0 * std_dev.powi(2));
        (x_val - mean).powi(2) / std_dev.powi(3) * exponent.exp()
    })
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Fit {
    pub histogram: Option<Histogram>,
    pub markers: EguiFitMarkers,
    pub background_line: Option<(f64, f64)>, // (slope, intercept)
    pub fit_line: Option<(f64, f64, f64)>, // (amplitude, mean, std_dev)
}

impl Fit {
    pub fn new() -> Self {
        Self {
            histogram: None,
            markers: EguiFitMarkers::new(),
            background_line: None,
            fit_line: None 
        }
    }

    // Background
    pub fn get_background_marker_data(&self) -> Vec<(f64, f64)> {

        if self.markers.background_markers.len() < 2 {
            log::info!("There are less than 2 background markers. Cannot generate background marker data.");
            return Vec::new();
        }

        let mut points = Vec::new();
        
        if let Some(histogram) = &self.histogram {
            log::debug!("Histogram is set, processing background markers.");
            for &x in &self.markers.background_markers {
                match histogram.get_bin(x) {
                    Some(bin_index) if bin_index < histogram.bins.len() => {
                        let y = histogram.bins[bin_index] as f64;
                        points.push((x, y));
                        log::trace!("Added point: ({}, {}) for bin index: {}", x, y, bin_index);
                    },
                    Some(bin_index) => {
                        // Bin index is out of bounds, which shouldn't normally happen
                        log::warn!("Bin index {} is out of bounds for background marker at x = {}. Ignoring this marker.", bin_index, x);
                    },
                    None => {
                        // x value doesn't correspond to a valid bin; it might be outside the histogram's range
                        log::warn!("No bin found for background marker at x = {}. This marker is outside the histogram's range.", x);
                    }
                }
            }
        } else {
            log::info!("No histogram set. Cannot process background markers.");
        }

        if points.is_empty() {
            log::info!("No background marker data was generated.");
        } else {
            log::debug!("Generated background marker data for {} points.", points.len());
        }

        points
    }

    pub fn perform_linear_fit_for_background(&mut self) -> Result<(), &'static str> {
        let background_marker_data = self.get_background_marker_data();
        if background_marker_data.is_empty() {
            return Err("No background markers set, cannot perform linear fit.");
        }

        let (x_values, y_values): (Vec<f64>, Vec<f64>) = background_marker_data.iter().cloned().unzip();

        match simple_linear_regression(&x_values, &y_values) {
            Ok((slope, intercept)) => {
                // Store the slope and intercept in self.background_line
                self.background_line = Some((slope, intercept));

                // Optionally, log the slope and intercept for debugging purposes
                log::info!("Background Fit (linear): slope: {}, intercept: {}", slope, intercept);

                Ok(())
            },
            Err(e) => Err(e),
        }
    }
    
    pub fn draw_background_line(&mut self, plot_ui: &mut PlotUi) {
        if let Some((slope, intercept)) = self.background_line {
            // Ensure there are background markers before proceeding
            if self.markers.background_markers.is_empty() {
                return;
            }
    
            // Find the minimum and maximum x-values among the background markers
            let min_x = self.markers.background_markers.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_x = self.markers.background_markers.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
            // Calculate the y-values for the line at these x-values
            let y1 = slope * min_x + intercept;
            let y2 = slope * max_x + intercept;
    
            // Create a vector of PlotPoints for the line between these points
            let plot_points = PlotPoints::Owned(vec![
                PlotPoint::new(min_x, y1),
                PlotPoint::new(max_x, y2),
            ]);
    
            // Define the line's appearance
            let color = Color32::GREEN;
            let line = Line::new(plot_points)
                .color(color)
                .stroke(Stroke::new(1.0, color));
    
            // Draw the line on the plot
            plot_ui.line(line);
        }
    }

    pub fn clear_background_line(&mut self) {
        self.background_line = None;
    }

    pub fn create_background_subtracted_histogram(&mut self) -> Result<Histogram, &'static str> {
        if self.background_line.is_none() {
            return Err("Background line has not been calculated.");
        }
        
        let (slope, intercept) = self.background_line.unwrap();

        // Ensure there's a histogram to subtract the background from
        if let Some(ref histogram) = self.histogram {
            let mut subtracted_histogram = histogram.clone(); // Clone the original histogram structure

            // Iterate through each bin in the histogram
            for (index, bin_count) in subtracted_histogram.bins.iter_mut().enumerate() {
                let bin_center = histogram.range.0 + (histogram.bin_width * index as f64) + (histogram.bin_width / 2.0);
                let background_estimate = slope * bin_center + intercept; // Calculate background count estimate for this bin
                
                // Subtract the background estimate from the bin count, ensuring it doesn't go below 0
                *bin_count = bin_count.saturating_sub(background_estimate as u32);
            }

            Ok(subtracted_histogram) // Return the background-subtracted histogram
        } else {
            Err("No histogram data available for background subtraction.")
        }
    }

    pub fn fit_gaussian(&mut self) {

        self.fit_line = None;
        
        let fit_region = self.markers.region_markers.clone();

        // Ensure we have exactly two region markers to define a fit region
        if fit_region.len() != 2 {
            log::error!("Need two region markers to define a fit region.");
            return;
        }

        // Ensure there are background markers; if not, use the region markers as defaults
        if self.markers.background_markers.is_empty() {
            self.markers.background_markers.push(fit_region[0]);
            self.markers.background_markers.push(fit_region[1]);
        }

        // Attempt to perform background subtraction and linear fit
        if let Err(e) = self.perform_linear_fit_for_background() {
            log::error!("Failed to perform linear fit for background: {}", e);
            return;
        }

        if let Ok(bg_sub_histogram) = self.create_background_subtracted_histogram() { 

            // Get bin indices from region markers
            let start_bin = bg_sub_histogram.get_bin(fit_region[0]).unwrap_or(0);
            let end_bin = bg_sub_histogram.get_bin(fit_region[1]).unwrap_or(bg_sub_histogram.bins.len() - 1);

            // Vector to store (bin_center, count) pairs
            let mut bin_data: Vec<(f64, u32)> = Vec::new();

            // Iterate over the range of bins
            for bin_index in start_bin..=end_bin {
                // Get the center of each bin
                let bin_center = bg_sub_histogram.bin_centers()[bin_index];
                
                // Get the count for each bin
                let count = bg_sub_histogram.bins[bin_index];

                bin_data.push((bin_center, count));
            }

            
            // Create DVector for x and y data for varpro fitting
            let x: DVector<f64> = DVector::from_column_slice(&bin_data.iter().map(|(x, _)| *x).collect::<Vec<f64>>());
            let y: DVector<f64> = DVector::from_column_slice(&bin_data.iter().map(|(_, y)| *y as f64).collect::<Vec<f64>>());


            if x.len() == 0 || y.len() == 0 {
                log::error!("No data available for Gaussian fit.");
                return;
            }

            let max: f64 = y.max();
            let max_index: usize = y.iter().position(|&x| x == max).unwrap();
    
            let mean: f64 = x[max_index];
            let std_dev: f64 = 1.0;
    
            let initial_guess = vec![mean, std_dev];
    
            let model = SeparableModelBuilder::<f64>::new(&["mean", "std_dev"])
                .initial_parameters(initial_guess)
                .independent_variable(x)
                .function(&["mean", "std_dev"],gaussian)
                .partial_deriv("mean", gaussian_pd_mean)
                .partial_deriv("std_dev", gaussian_pd_std_dev)
                .build()
                .unwrap();
    
            let problem = LevMarProblemBuilder::new(model)
                .observations(y)
                .build()
                .unwrap();
    
            // fit the data
            let fit_result = LevMarSolver::default()
                            .fit(problem)
                            .expect("fit must succeed");
    
            // the nonlinear parameters
            let alpha = fit_result.nonlinear_parameters();
            // the linear coefficients
            let c  = fit_result.linear_coefficients().unwrap();
    
            log::info!("Fitted Gaussian: mean: {}, std_dev: {}", alpha[0], alpha[1]);
            log::info!("Linear coefficients: {:?}", c);
    
            self.fit_line = Some((c[0], alpha[0], alpha[1]));

        }

    }

    pub fn draw_fit(&self, plot_ui: &mut PlotUi) {
        if let Some((amplitude, mean, std_dev)) = self.fit_line {
            let num_points = 100;
            let start = self.markers.region_markers.iter().cloned().fold(f64::INFINITY, f64::min);
            let end = self.markers.region_markers.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let step = (end - start) / num_points as f64;
    
            let plot_points: Vec<PlotPoint> = (0..=num_points).map(|i| {
                let x = start + step * i as f64;
                let y = amplitude * (-(x - mean).powi(2) / (2.0 * std_dev.powi(2))).exp();
                PlotPoint::new(x, y)
            }).collect();
    
            let line = Line::new(PlotPoints::Owned(plot_points))
                .color(Color32::RED)
                .stroke(Stroke::new(2.0, Color32::RED))
                .name("Gaussian Fit");
    
            plot_ui.line(line);
        }
    }

    pub fn clear_fit_line(&mut self) {
        self.fit_line = None;
    }

    pub fn interactive_fitter(&mut self, ui: &mut egui::Ui) {
        
        self.markers.interactive_markers(ui);

        if ui.input(|i| i.key_pressed(egui::Key::F)) {

            self.fit_gaussian();
        }

        ui.horizontal(|ui| {

            ui.label("Fitter");
            
            ui.separator();

            ui.label("Clear: ");

            if ui.button("Peaks").on_hover_text("Clear all peak markers").clicked() {
                self.markers.clear_peak_markers();
            }

            if ui.button("Background").on_hover_text("Clear all background markers").clicked() {
                self.markers.clear_background_markers();
                self.clear_background_line();
            }

            if ui.button("Region").on_hover_text("Clear all region markers").clicked() {
                self.markers.clear_region_markers();
            }

            ui.separator();

            if ui.button("Clear all").on_hover_text("Clear all markers").clicked() {
                self.markers.clear_peak_markers();
                self.markers.clear_background_markers();
                self.markers.clear_region_markers();
                self.clear_background_line();
                self.clear_fit_line();
            }
        });

    }

    // draw fit lines
    pub fn draw_fit_lines(&mut self, plot_ui: &mut PlotUi) {
        self.draw_background_line(plot_ui);
        self.draw_fit(plot_ui);
    }

}


pub fn simple_linear_regression(x_data: &[f64], y_data: &[f64]) -> Result<(f64, f64), &'static str> {

    if x_data.len() != y_data.len() || x_data.is_empty() {
        return Err("x_data and y_data must have the same non-zero length");
    }

    let n = x_data.len() as f64;
    let sum_x = x_data.iter().sum::<f64>();
    let sum_y = y_data.iter().sum::<f64>();
    let sum_xy = x_data.iter().zip(y_data.iter()).map(|(x, y)| x * y).sum::<f64>();
    let sum_x_squared = x_data.iter().map(|x| x.powi(2)).sum::<f64>();
    
    let denominator = n * sum_x_squared - sum_x.powi(2);
    if denominator == 0.0 {
        return Err("Denominator in slope calculation is zero, cannot compute slope and intercept");
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denominator;
    let intercept = (sum_y - slope * sum_x) / n;

    Ok((slope, intercept))
}

    /* 
pub struct GaussianFitter {
    number_of_peaks: usize,
    same_std_dev: bool,
    hold_peak_positions: bool,
    hold_peak_widths: bool,
    x: DVector<f64>,
    y: DVector<f64>,
}

impl GaussianFitter {
    pub fn new(x: DVector<f64>, y: DVector<f64>) -> Self {
        Self {
            number_of_peaks: 1,
            same_std_dev: true,
            hold_peak_positions: false,
            hold_peak_widths: false,
            x,
            y,
        }
    }

    fn gaussian(x: &DVector<f64>, mean: f64, std_dev: f64) -> DVector<f64> {
        x.map(|x_val| (-((x_val - mean).powi(2)) / (2.0 * std_dev.powi(2))).exp())
    }
    
    fn gaussian_pd_mean(x: &DVector<f64>, mean: f64, std_dev: f64) -> DVector<f64> {
        x.map(|x_val| (x_val - mean) / std_dev.powi(2) * (-((x_val - mean).powi(2)) / (2.0 * std_dev.powi(2))).exp())
    }
    
    fn gaussian_pd_std_dev(x: &DVector<f64>, mean: f64, std_dev: f64) -> DVector<f64> {
        x.map(|x_val| {
            let exponent = -((x_val - mean).powi(2)) / (2.0 * std_dev.powi(2));
            (x_val - mean).powi(2) / std_dev.powi(3) * exponent.exp()
        })
    }

    pub fn set_number_of_peaks(&mut self, number_of_peaks: usize) {
        self.number_of_peaks = number_of_peaks;
    }

}*/
