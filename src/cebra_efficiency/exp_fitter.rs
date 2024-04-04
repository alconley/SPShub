use nalgebra::DVector;
use varpro::model::builder::SeparableModelBuilder;
use varpro::solvers::levmar::{LevMarProblemBuilder, LevMarSolver};

use egui::{Color32, Stroke};
use egui_plot::{Line, PlotPoint, PlotPoints, PlotUi};

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct ExpFitter {
    pub fit_params: Option<Vec<((f64, f64), (f64, f64))>>,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub weights: Vec<f64>,

    #[serde(skip)]
    pub fit_line: Option<Vec<Vec<PlotPoint>>>,

    pub fit_label: String,
}

impl ExpFitter {
    pub fn new(x: Vec<f64>, y: Vec<f64>, weights: Vec<f64>) -> Self {
        Self {
            fit_params: None,
            x,
            y,
            weights,
            fit_line: None,
            fit_label: "".to_string(),
        }
    }

    fn exponential(x: &DVector<f64>, b: f64) -> DVector<f64> {
        x.map(|x_val| (-x_val / b).exp())
    }

    fn exponential_pd_b(x: &DVector<f64>, b: f64) -> DVector<f64> {
        x.map(|x_val| (x_val / b.powi(2)) * (-x_val / b).exp())
    }

    fn exponential_pd_d(x: &DVector<f64>, d: f64) -> DVector<f64> {
        x.map(|x_val| (x_val / d.powi(2)) * (-x_val / d).exp())
    }

    pub fn single_exp_fit(&mut self) {
        self.fit_params = None;
        self.fit_line = None;
        self.fit_label = "".to_string();

        let x_data = DVector::from_vec(self.x.clone());
        let y_data = DVector::from_vec(self.y.clone());
        let weights = DVector::from_vec(self.weights.clone());

        let parameter_names: Vec<String> = vec!["b".to_string()];
        let initial_guess: Vec<f64> = vec![100.0];

        let builder_proxy = SeparableModelBuilder::<f64>::new(parameter_names)
            .initial_parameters(initial_guess)
            .independent_variable(x_data)
            .function(&["b"], Self::exponential)
            .partial_deriv("b", Self::exponential_pd_b);

        let model = builder_proxy.build().unwrap();

        let problem = LevMarProblemBuilder::new(model)
            .observations(y_data)
            .weights(weights)
            .build()
            .unwrap();

        if let Ok((fit_result, fit_statistics)) =
            LevMarSolver::default().fit_with_statistics(problem)
        {
            log::info!(
                "Nonlinear Parameters: {:?}",
                fit_result.nonlinear_parameters()
            );
            log::info!(
                "nonlinear parameters variance: {:?}",
                fit_statistics.nonlinear_parameters_variance()
            );

            log::info!(
                "Linear Coefficients: {:?}",
                fit_result.linear_coefficients().unwrap()
            );
            log::info!(
                "linear coefficients variance: {:?}",
                fit_statistics.linear_coefficients_variance()
            );

            let nonlinear_parameters = fit_result.nonlinear_parameters();
            let nonlinear_variances = fit_statistics.nonlinear_parameters_variance();

            let linear_coefficients = fit_result.linear_coefficients().unwrap();
            let linear_variances = fit_statistics.linear_coefficients_variance();

            let parameter_a = linear_coefficients[0];
            let parameter_a_variance = linear_variances[0];
            let parameter_a_uncertainity = parameter_a_variance.sqrt();

            let parameter_b = nonlinear_parameters[0];
            let parameter_b_variance = nonlinear_variances[0];
            let parameter_b_uncertainity = parameter_b_variance.sqrt();

            let fit_string = format!(
                "Y = ({:.2} ± {:.2}) * exp[ -x / ({:.2} ± {:.2}) ]",
                parameter_a, parameter_a_uncertainity, parameter_b, parameter_b_uncertainity
            );
            self.fit_label = fit_string;

            let parameters = vec![(
                (parameter_a, parameter_a_uncertainity),
                (parameter_b, parameter_b_uncertainity),
            )];
            log::info!("parameters: {:?}", parameters);

            self.fit_params = Some(parameters);
        }
    }

    pub fn double_exp_fit(&mut self) {
        self.fit_params = None;
        self.fit_line = None;
        self.fit_label = "".to_string();

        let x_data = DVector::from_vec(self.x.clone());
        let y_data = DVector::from_vec(self.y.clone());
        let weights = DVector::from_vec(self.weights.clone());

        let parameter_names: Vec<String> = vec!["b".to_string(), "d".to_string()];
        let initial_guess: Vec<f64> = vec![100.0, 100.0];

        let builder_proxy = SeparableModelBuilder::<f64>::new(parameter_names)
            .initial_parameters(initial_guess)
            .independent_variable(x_data)
            .function(&["b"], Self::exponential)
            .partial_deriv("b", Self::exponential_pd_b)
            .function(&["d"], Self::exponential)
            .partial_deriv("d", Self::exponential_pd_d);

        let model = builder_proxy.build().unwrap();

        let problem = LevMarProblemBuilder::new(model)
            .observations(y_data)
            .weights(weights)
            .build()
            .unwrap();

        if let Ok((fit_result, fit_statistics)) =
            LevMarSolver::default().fit_with_statistics(problem)
        {
            log::info!(
                "Nonlinear Parameters: {:?}",
                fit_result.nonlinear_parameters()
            );
            log::info!(
                "nonlinear parameters variance: {:?}",
                fit_statistics.nonlinear_parameters_variance()
            );

            log::info!(
                "Linear Coefficients: {:?}",
                fit_result.linear_coefficients().unwrap()
            );
            log::info!(
                "linear coefficients variance: {:?}",
                fit_statistics.linear_coefficients_variance()
            );

            let nonlinear_parameters = fit_result.nonlinear_parameters();
            let nonlinear_variances = fit_statistics.nonlinear_parameters_variance();

            let linear_coefficients = fit_result.linear_coefficients().unwrap();
            let linear_variances = fit_statistics.linear_coefficients_variance();

            let parameter_a = linear_coefficients[0];
            let parameter_a_variance = linear_variances[0];
            let parameter_a_uncertainity = parameter_a_variance.sqrt();

            let parameter_b = nonlinear_parameters[0];
            let parameter_b_variance = nonlinear_variances[0];
            let parameter_b_uncertainity = parameter_b_variance.sqrt();

            let exp_1 = (
                (parameter_a, parameter_a_uncertainity),
                (parameter_b, parameter_b_uncertainity),
            );

            let parameter_c = linear_coefficients[1];
            let parameter_c_variance = linear_variances[1];
            let parameter_c_uncertainity = parameter_c_variance.sqrt();

            let parameter_d = nonlinear_parameters[1];
            let parameter_d_variance = nonlinear_variances[1];
            let parameter_d_uncertainity = parameter_d_variance.sqrt();

            let exp_2 = (
                (parameter_c, parameter_c_uncertainity),
                (parameter_d, parameter_d_uncertainity),
            );

            let parameters = vec![exp_1, exp_2];

            let fit_string = format!("Y = ({:.2} ± {:.2}) * exp[ -x / ({:.2}±{:.2}) ] + ({:.2} ± {:.2}) * exp[ -x / ({:.2} ± {:.2}) ]",
                parameter_a, parameter_a_uncertainity,
                parameter_b, parameter_b_uncertainity,
                parameter_c, parameter_c_uncertainity,
                parameter_d, parameter_d_uncertainity);

            self.fit_label = fit_string;

            self.fit_params = Some(parameters);
        }
    }

    pub fn fit_ui(&mut self, ui: &mut egui::Ui) {
        ui.separator();

        ui.label("Exponential Fitter:");

        if ui.button("Single").clicked() {
            self.single_exp_fit();
        }

        if ui.button("Double").clicked() {
            self.double_exp_fit();
        }

        ui.separator();

        ui.label(self.fit_label.clone());
    }
}
