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

    // fn exponential()
}