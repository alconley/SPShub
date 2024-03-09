use egui_plot::{Line, PlotUi, PlotPoint, PlotPoints};
use egui::{Color32, Stroke};

pub struct BackgroundFitter {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub background_params: Option<(f64, f64)>, // (slope, intercept)
    pub background_line_points: Option<Vec<PlotPoint>>, 
}

impl BackgroundFitter {
    //currently only for a linear fit
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> Self {
        Self {
            x,
            y,
            background_params: None,
            background_line_points: None,
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
    
    pub fn perform_linear_fit_for_background(&mut self) -> Result<(), &'static str> {
        // Ensure there's data to perform linear regression on
        if self.x.is_empty() || self.y.is_empty() {
            return Err("Insufficient data for linear regression.");
        }

        match Self::simple_linear_regression(&self.x, &self.y) {
            Ok((slope, intercept)) => {
                self.background_params = Some((slope, intercept));
                log::info!("Background Fit (linear): slope: {}, intercept: {}", slope, intercept);
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    pub fn calculate_background_line_points(&mut self) {
        if let Some((slope, intercept)) = self.background_params {
            let min_x = *self.x.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let max_x = *self.x.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

            let y1 = slope * min_x + intercept;
            let y2 = slope * max_x + intercept;

            let plot_points = vec![
                PlotPoint::new(min_x, y1),
                PlotPoint::new(max_x, y2),
            ];

            self.background_line_points = Some(plot_points);
        }
    }

    pub fn fit(&mut self) -> Result<(), &'static str> {
        self.perform_linear_fit_for_background()?;
        self.calculate_background_line_points();
        Ok(())
    }

    pub fn draw_background_line(&self, plot_ui: &mut PlotUi) {
        if let Some(plot_points) = &self.background_line_points {
            // Clone `plot_points` to satisfy the ownership requirement of `Line::new`
            let plot_points_cloned = plot_points.clone();
            
            // Define the line's appearance
            let color = Color32::GREEN;
            let line = Line::new(PlotPoints::Owned(plot_points_cloned))
                .color(color)
                .stroke(Stroke::new(1.0, color));
    
            plot_ui.line(line);
        }
    }

}


// pub fn get_background_marker_data(&self, histogram: &Histogram) -> Vec<(f64, f64)> {
//     let mut points = Vec::new();
    
//     if let Some(bg_markers) = &self.background_markers {
//         for &x in bg_markers {
//             if let Some(bin_index) = histogram.get_bin(x) {
//                 let bin_center = histogram.range.0 + (bin_index as f64 * histogram.bin_width) + (histogram.bin_width * 0.5);
//                 points.push((bin_center, histogram.bins[bin_index] as f64));
//             }
//         }
//     }

//     points
// }


// pub fn create_background_subtracted_histogram(&self, histogram: &Histogram) -> Result<Histogram, &'static str> {
//     let (slope, intercept) = self.background_params.ok_or("Background parameters not set.")?;

//     let mut subtracted_histogram = histogram.clone();

//     // Subtract background estimate from each bin
//     for (index, bin_count) in subtracted_histogram.bins.iter_mut().enumerate() {
//         let bin_center = histogram.range.0 + (histogram.bin_width * index as f64) + (histogram.bin_width / 2.0);
//         let background_estimate = slope * bin_center + intercept;
//         *bin_count = bin_count.saturating_sub(background_estimate.round() as u32);
//     }

//     Ok(subtracted_histogram)
// }