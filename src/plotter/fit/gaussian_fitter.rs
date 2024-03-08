use varpro::solvers::levmar::{LevMarProblemBuilder, LevMarSolver};
use varpro::model::builder::SeparableModelBuilder;
use nalgebra::DVector;

use egui_plot::{Line, PlotUi, PlotPoint, PlotPoints};
use egui::{Color32, Stroke};

use super::egui_markers::EguiFitMarkers;

pub struct GaussianFitter {
    x: DVector<f64>,
    y: DVector<f64>,
    markers: EguiFitMarkers,
    fit_with_same_sigma: bool,
    fit_params: Vec<(f64, f64, f64)>,
    convoluted_fit_line: Line,
    decomposition_fit_lines: Vec<Line>
}

impl GaussianFitter {
    pub fn new(x: DVector<f64>, y: DVector<f64>, markers: EguiFitMarkers) -> Self {
        Self {
            x,
            y,
            markers,
            fit_with_same_sigma: true,
            fit_params: Vec::new(),
            convoluted_fit_line: Line::new(PlotPoints::Owned(Vec::new())),
            decomposition_fit_lines: Vec::new()
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

    pub fn initial_guess(&self) -> Vec<f64> {
        let mut initial_guesses: Vec<f64> = Vec::new();
        // Add means based on peak_markers

        if self.fit_with_same_sigma {
            for &mean in &self.markers.peak_markers {
                initial_guesses.push(mean);
            }
            if self.markers.peak_markers.len() == 0 {
                initial_guesses.push(self.x.mean());
            }

            initial_guesses.push(1.0);
        } else {
            for &mean in &self.markers.peak_markers {
                initial_guesses.push(mean);
                initial_guesses.push(1.0);
            }

            if self.markers.peak_markers.len() == 0 {
                initial_guesses.push(self.x.mean());
                initial_guesses.push(1.0);
            }
        }

        initial_guesses
        
    }

    pub fn multi_gauss_fit(&self) {
        self.fit_params = Vec::new();

        // Assuming `self.markers.peak_markers` determines the number of peaks and
        // `self.initial_guess()`, `self.x`, etc., provide necessary data.
        let initial_guess = self.initial_guess();
        let x_data = self.x.clone();

        let mut parameter_names = Vec::new();
        // Generate parameter names for all peaks
        for i in 0..self.markers.peak_markers.len() {
            parameter_names.push(format!("mean{}", i));
        }
        parameter_names.push("std_dev".to_string()); // Add a single standard deviation parameter at the end

        // Add parameters for the first peak manually
        let mut builder_proxy = SeparableModelBuilder::<f64>::new(&parameter_names)
            .initial_parameters(initial_guess)
            .independent_variable(x_data)
            .function(&["mean0", "std_dev"], Self::gaussian)
            .partial_deriv("mean0", Self::gaussian_pd_mean)
            .partial_deriv("std_dev", Self::gaussian_pd_std_dev);

        // Now, iterate starting from the second peak since the first peak is already handled
        for i in 1..self.markers.peak_markers.len() {
            let peak_parameters = vec![format!("mean{}", i), "std_dev".to_string()];
            
            // For each subsequent peak, add the function and its derivatives
            builder_proxy = builder_proxy
                .function(&peak_parameters, Self::gaussian)
                .partial_deriv(format!("mean{}", i), Self::gaussian_pd_mean)
                .partial_deriv("std_dev", Self::gaussian_pd_std_dev);
        }

        // Finalize the model building process
        let model = builder_proxy.build().unwrap();

        // extract the parameters
        let problem = LevMarProblemBuilder::new(model)
            .observations(self.y.clone())
            .build()
            .unwrap();

        let fit_result = LevMarSolver::default()
            .fit(problem)
            .expect("fit must succeed");

        // the nonlinear parameters
        let alpha = fit_result.nonlinear_parameters();

        // the linear coefficients
        let c  = fit_result.linear_coefficients().unwrap();

        // clear peak markers and update the markers with the centroid from the fit
        self.markers.peak_markers.clear();
        for i in 0..alpha.nrows() - 1 {
            self.markers.peak_markers.push(alpha[(i, 0)]);
        }

        // create a vec to store the fit parameters for each gaussian (c, mean, std_dev)
        let mut fit_params: Vec<(f64, f64, f64)> = Vec::new();

        let std_dev_index = alpha.nrows() - 1; // Assuming a column vector; use ncols() if a row vector.
        let std_dev = alpha[(std_dev_index, 0)]; 
        
        // Iterate over the means and coefficients together
        for (i, &mean) in alpha.iter().enumerate().take(self.markers.peak_markers.len()) {
            let coefficient = c[i]; // Corresponding coefficient for this peak

            // Store the parameters for each peak
            fit_params.push((coefficient, mean, std_dev));
        }

        // Logging or further processing with `fit_params`
        for (index, (coefficient, mean, std_dev)) in fit_params.iter().enumerate() {
            log::info!("Peak {}: Coefficient: {:.2}, Mean: {:.2}, Std Dev: {:.2}", index, coefficient, mean, std_dev);
        }

        self.fit_params = fit_params

    }

    pub fn fit_decomposition_lines(&self) {
        self.decomposition_fit_lines = Vec::new();

        for (index, (coefficient, mean, std_dev)) in self.fit_params.iter().enumerate() {
            let num_points = 100;
    
            // Adjust start and end to be 5 sigma from the mean
            let start = mean - 5.0 * std_dev;
            let end = mean + 5.0 * std_dev;
            let step = (end - start) / num_points as f64;
    
            let plot_points: Vec<PlotPoint> = (0..=num_points).map(|i| {
                let x = start + step * i as f64;
                let y = coefficient * (-(x - mean).powi(2) / (2.0 * std_dev.powi(2))).exp();
                PlotPoint::new(x, y)
            }).collect();
    
            let fit_name = format!("Fit {}", index); // Generate a name for each fit line
    
            let line = Line::new(PlotPoints::Owned(plot_points))
                .color(Color32::RED)
                .stroke(Stroke::new(1.0, Color32::BLUE))
                .name(&fit_name); // Use the generated name here
    
            self.decomposition_fit_lines.push(line);
        }

    }

    pub fn convoluted_fit_line(&mut self) {
        let num_points = 1000; // Number of points to generate for the fit line
        let min_x = self.x.min(); // Minimum x-value, you may adjust based on your data range
        let max_x = self.x.max(); // Maximum x-value, you may adjust based on your data range
        let step = (max_x - min_x) / num_points as f64; // Step size to generate points
        
        let mut plot_points: Vec<PlotPoint> = Vec::new();

        // Generate points for the convoluted fit line
        for i in 0..=num_points {
            let x_val = min_x + step * i as f64;
            // Sum up contributions from all Gaussian peaks at this x-value
            let y_val = self.fit_params.iter().fold(0.0, |acc, &(amplitude, mean, std_dev)| {
                acc + amplitude * (-((x_val - mean).powi(2)) / (2.0 * std_dev.powi(2))).exp()
            });

            plot_points.push(PlotPoint::new(x_val, y_val));
        }

        // Define the line's appearance
        let color = Color32::YELLOW; // Example color, adjust as needed
        let convoluted_fit_line = Line::new(PlotPoints::Owned(plot_points))
            .color(color)
            .stroke(Stroke::new(2.0, color))
            .name("Convoluted Fit"); // Name of the convoluted fit line

        // Clear previous convoluted fit lines and add the new one
        self.convoluted_fit_line = convoluted_fit_line;
    }

}