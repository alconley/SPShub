use egui_plot::{Line, PlotUi, PlotPoint, PlotPoints};
use egui::{Color32, Stroke};

use crate::plotter::histogram1d::Histogram;

pub struct BackgroundFitter {
    pub background_markers: Option<Vec<f64>>,
    pub background_params: Option<(f64, f64)>,
    pub background_line: Option<Line>,
}

impl BackgroundFitter {
    //currently only for a linear fit
    pub fn new() -> Self {
        Self {
            background_markers: None,
            background_params: None,
            background_line: None,
        }
    }
        
    fn simple_linear_regression(x_data: &[f64], y_data: &[f64]) -> Result<(f64, f64), &'static str> {

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

    fn get_background_marker_data(&self, histogram: Histogram) -> Vec<(f64, f64)> {
        let mut points = Vec::new();
        
        // Ensure background_markers is Some before proceeding
        if let Some(bg_markers) = &self.background_markers {
            for &x in bg_markers {
                if let Some(bin_index) = histogram.get_bin(x) {
                    // Calculate the center of the bin
                    let bin_center = histogram.range.0 + (bin_index as f64 * histogram.bin_width) + (histogram.bin_width * 0.5);
                    // Add the bin center and its count to the points vector
                    points.push((bin_center, histogram.bins[bin_index] as f64));
                }
            }
        }

        points
    }

    fn perform_linear_fit_for_background(&mut self, histogram: Histogram) -> Result<(), &'static str> {
        if self.background_markers.is_none() || self.background_markers.as_ref().unwrap().is_empty() {
            return Err("No background markers set, cannot perform linear fit.");
        }

        let background_marker_data = self.get_background_marker_data(histogram);

        if background_marker_data.is_empty() {
            return Err("Failed to retrieve background marker data.");
        }

        let (x_values, y_values): (Vec<f64>, Vec<f64>) = background_marker_data.iter().cloned().unzip();

        match Self::simple_linear_regression(&x_values, &y_values) {
            Ok((slope, intercept)) => {
                // Store the slope and intercept
                self.background_params = Some((slope, intercept));
                log::info!("Background Fit (linear): slope: {}, intercept: {}", slope, intercept);
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    fn create_background_subtracted_histogram(&self, histogram: &Histogram) -> Result<Histogram, &'static str> {
        let (slope, intercept) = self.background_params.ok_or("Background parameters not set.")?;

        let mut subtracted_histogram = histogram.clone();

        // Subtract background estimate from each bin
        for (index, bin_count) in subtracted_histogram.bins.iter_mut().enumerate() {
            let bin_center = histogram.range.0 + (histogram.bin_width * index as f64) + (histogram.bin_width / 2.0);
            let background_estimate = slope * bin_center + intercept;
            *bin_count = bin_count.saturating_sub(background_estimate.round() as u32);
        }

        Ok(subtracted_histogram)
    }

    fn calculate_background_line(&mut self)-> Option<Line>  {
        // Ensure there are background markers and they are not empty before proceeding
        if let Some(bg_markers) = &self.background_markers {
            if bg_markers.is_empty() || self.background_params.is_none() {
                return None;
            }

            // Extract the slope and intercept for the linear background
            let (slope, intercept) = self.background_params.unwrap();

            // Find the minimum and maximum x-values among the background markers
            let min_x = bg_markers.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_x = bg_markers.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            // Calculate the y-values for the line at these x-values
            let y1 = slope * min_x + intercept;
            let y2 = slope * max_x + intercept;

            // Create a vector of PlotPoints for the line between these points
            let plot_points = vec![
                PlotPoint::new(min_x, y1),
                PlotPoint::new(max_x, y2),
            ];

            // Define the line's appearance
            let color = Color32::GREEN; // You can adjust the color as needed
            let line = Line::new(PlotPoints::Owned(plot_points))
                .color(color)
                .stroke(Stroke::new(1.0, color)); // Adjust the stroke as needed

            Some(line)
        } else {
            None
        }

    }

    pub fn update_background_line(&mut self) {
        // Call calculate_background_line and update self.background_line accordingly
        self.background_line = self.calculate_background_line();
    }

    pub fn clear_background_line(&mut self) {
        self.background_line = None;
    }

    pub fn draw_background_line(&mut self, plot_ui: &mut PlotUi) {

        if let Some(line) = &self.background_line {
            plot_ui.line(*line);
        }
    }

}
