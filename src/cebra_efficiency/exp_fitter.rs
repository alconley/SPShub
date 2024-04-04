use nalgebra::DVector;
use varpro::model::builder::SeparableModelBuilder;
use varpro::solvers::levmar::{LevMarProblemBuilder, LevMarSolver};

pub struct ExpFitter {
    pub fit_params: Option<Vec<f64>>,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub weights: Vec<f64>,
}

impl ExpFitter {
    pub fn new(x: Vec<f64>, y: Vec<f64>, weights: Vec<f64>) -> Self {
        Self {
            fit_params: None,
            x,
            y,
            weights,
        }
    }

    fn exponential(x: &DVector<f64>, b: f64) -> DVector<f64> {
        x.map(|x_val| (-x_val/b).exp())
    }

    fn exponential_pd_b(x: &DVector<f64>, b: f64) -> DVector<f64> {
        x.map(|x_val| (x_val/b.powi(2) ) * (-x_val/b).exp())
    }

    fn fit(&mut self) {
        let x_data = DVector::from_vec(self.x.clone());
        let y_data = DVector::from_vec(self.y.clone());
        let weights = DVector::from_vec(self.weights.clone());

        let parameter_names: Vec<String> = vec!["b".to_string()];
        let initial_guess: Vec<f64> = vec![100.0];

        let mut builder_proxy = SeparableModelBuilder::<f64>::new(parameter_names)
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

        }
    }
}