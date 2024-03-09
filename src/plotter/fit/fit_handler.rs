use egui_plot::PlotUi;

use egui_plot::{Line, PlotPoint, PlotPoints};
use egui::{Color32, Stroke};

use super::egui_markers::EguiFitMarkers;
use super::gaussian_fitter::GaussianFitter;
use super::background_fitter::BackgroundFitter;

use crate::plotter::histogram1d::Histogram;

#[derive(Default)]
pub struct FitHandler {
    pub histogram: Option<Histogram>,
    pub fits: Vec<Fit>,
    pub current_fit: Option<Fit>,
    pub markers: EguiFitMarkers,
}

impl FitHandler {
    pub fn new() -> Self {
        Self {
            histogram: None,
            fits: Vec::new(),
            current_fit: None,
            markers: EguiFitMarkers::new(),
        }
    }

    pub fn interactive_keybinds(&mut self, ui: &mut egui::Ui) {

        if ui.input(|i| i.key_pressed(egui::Key::Minus)) {

            if let Some(fit) = &mut self.current_fit {
                fit.clear();
            }

            self.markers.delete_closest_marker();
        }
        
        self.markers.interactive_markers(ui);

        if ui.input(|i| i.key_pressed(egui::Key::F)) {
            if let Some(histogram) = self.histogram.clone() {
                self.new_fit(histogram);
            } else {
                eprintln!("No histogram selected for fitting.");
            }
        }

        if ui.input(|i| i.key_pressed(egui::Key::S)) {
            self.store_fit();
        }

        if ui.input(|i| i.key_pressed(egui::Key::Backspace)) {

            if let Some(fit) = &mut self.current_fit {
                fit.clear();
            }

            self.markers.clear_background_markers();
            self.markers.clear_peak_markers();
            self.markers.clear_region_markers();

        }
        
    }


    fn new_fit(&mut self, histogram: Histogram) {
        let mut fit = Fit::new(histogram, self.markers.clone());

        if let Err(e) = fit.fit_gaussian() {
            eprintln!("Failed to fit gaussian: {}", e);
        }

        self.markers = fit.markers.clone(); // update the makers with the fit markers
        self.current_fit = Some(fit);

    }

    fn store_fit(&mut self) {
        if let Some(fit) = self.current_fit.take() {
            self.fits.push(fit);
        }
    }

    pub fn draw_fits(&mut self, plot_ui: &mut PlotUi) {
        
        // draw the current fit
        if let Some(fit) = &mut self.current_fit {
            fit.draw(plot_ui, Color32::BLUE);
        }

        // draw the stored fits
        for fit in &mut self.fits {
            let color = Color32::from_rgb(162, 0, 255);
            fit.draw(plot_ui, color);
        }
    }


}

pub struct Fit {
    histogram: Histogram,
    markers: EguiFitMarkers,
    fit: Option<GaussianFitter>,
    background: Option<BackgroundFitter>,
}

impl Fit {
    pub fn new(histogram: Histogram, markers: EguiFitMarkers) -> Self {
        Self {
            histogram,
            markers,
            fit: None,
            background: None,
        }
    }

    fn get_background_marker_data(&self) -> (Vec<f64>, Vec<f64>) {

        let bg_markers = self.markers.background_markers.clone();

        let mut y_values = Vec::new();
        let mut x_values = Vec::new();

        for x in bg_markers {
            // get the bin index
            if let Some(bin_index) = self.histogram.get_bin(x) {
                let bin_center = self.histogram.range.0 + (bin_index as f64 * self.histogram.bin_width) + (self.histogram.bin_width * 0.5);
                x_values.push(bin_center);
                y_values.push(self.histogram.bins[bin_index] as f64);
            }

        }

        (x_values, y_values)
    }

    fn fit_background(&mut self) -> Result<(), &'static str> {
        let (x_values, y_values) = self.get_background_marker_data();

        // Initialize BackgroundFitter with the obtained x and y values
        let mut background_fitter = BackgroundFitter::new(x_values, y_values);

        // Perform the fit and calculate background line points
        background_fitter.fit()?;

        // Update the background property with the fitted background_fitter
        self.background = Some(background_fitter);

        Ok(())
    }

    fn create_background_subtracted_histogram(&self) -> Result<Histogram, &'static str> {

        if let Some(background_fitter) = &self.background {
            let (slope, intercept) = background_fitter.background_params.ok_or("Background parameters not set.")?;

            let mut subtracted_histogram = self.histogram.clone();

            // Subtract background estimate from each bin
            for (index, bin_count) in subtracted_histogram.bins.iter_mut().enumerate() {
                let bin_center = self.histogram.range.0 + (self.histogram.bin_width * index as f64) + (self.histogram.bin_width / 2.0);
                let background_estimate = slope * bin_center + intercept;
                *bin_count = bin_count.saturating_sub(background_estimate.round() as u32);
            }

            Ok(subtracted_histogram)

        } else {
            Err("No background fitter available for background subtraction.")
        }
    }

    fn fit_gaussian(&mut self) -> Result<(), &'static str> {

        // Ensure there are exactly two region markers to define a fit region
        if self.markers.region_markers.len() != 2 {
            return Err("Need two region markers to define a fit region.");
        }

        // remove peak markers that are outside the region markers
        self.markers.remove_peak_markers_outside_region();

        // if there are no background markers, use the region markers as defaults
        if self.markers.background_markers.is_empty() {
            self.markers.background_markers.push(self.markers.region_markers[0]);
            self.markers.background_markers.push(self.markers.region_markers[1]);
        }

        // fit the background
        let _ = self.fit_background();

        // Ensure background subtraction has been performed
        let bg_subtracted_histogram = self.create_background_subtracted_histogram()?;

        // Extract x and y data between region markers
        let start_bin = bg_subtracted_histogram.get_bin(self.markers.region_markers[0]).unwrap_or(0);
        let end_bin = bg_subtracted_histogram.get_bin(self.markers.region_markers[1]).unwrap_or(bg_subtracted_histogram.bins.len() - 1);

        let mut x_data = Vec::new();
        let mut y_data = Vec::new();

        for bin_index in start_bin..=end_bin {
            let bin_center = bg_subtracted_histogram.range.0 + (bg_subtracted_histogram.bin_width * bin_index as f64) + (bg_subtracted_histogram.bin_width / 2.0);
            let bin_count = bg_subtracted_histogram.bins[bin_index];
            x_data.push(bin_center);
            y_data.push(bin_count as f64);
        }

        // Initialize GaussianFitter with x and y data
        let mut gaussian_fitter = GaussianFitter::new(x_data, y_data, self.markers.peak_markers.clone());

        // Perform Gaussian fit
        gaussian_fitter.multi_gauss_fit();

        // get the decomposition fit lines
        gaussian_fitter.get_fit_decomposition_line_points();

        // update peak markers with the fitted peak markers
        self.markers.peak_markers = gaussian_fitter.peak_markers.clone();

        // Update the fit property with the fitted GaussianFitter
        self.fit = Some(gaussian_fitter);

        Ok(())

    }

    fn draw(&mut self, plot_ui: &mut PlotUi, color: Color32) {
        if let Some(background_fitter) = &self.background {
            background_fitter.draw_background_line(plot_ui);

            if let Some(gaussian_fitter) = &self.fit {
                gaussian_fitter.draw_decomposition_fit_lines(plot_ui, color);
    
                let slope = background_fitter.background_params.unwrap().0;
                let intercept = background_fitter.background_params.unwrap().1;
    
                 // Calculate and draw the convoluted fit
                 let convoluted_fit_points = gaussian_fitter.calculate_convoluted_fit_points_with_background(slope, intercept);
                 let line = Line::new(PlotPoints::Owned(convoluted_fit_points))
                     .color(color) // Choose a distinct color for the convoluted fit
                     .stroke(Stroke::new(2.0, color));
                 plot_ui.line(line);
            }
        }   

    }

    fn clear(&mut self) {
        self.fit = None;
        self.background = None;
    }

}
