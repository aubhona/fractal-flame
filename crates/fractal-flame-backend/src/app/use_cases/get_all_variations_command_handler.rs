use std::sync::Arc;

use fractal_flame_core::domain::transformation::Transformation;

use super::get_all_variations_command::{
    GetAllVariationsCommand, GetAllVariationsCommandResult, VariationDto,
};

pub struct GetAllVariationsCommandHandler {
    pub transformations: Arc<Vec<Box<dyn Transformation + Send + Sync>>>,
}

impl GetAllVariationsCommandHandler {
    pub fn new(transformations: Arc<Vec<Box<dyn Transformation + Send + Sync>>>) -> Self {
        Self { transformations }
    }

    pub fn handle(&self, _command: GetAllVariationsCommand) -> GetAllVariationsCommandResult {
        let variations = self
            .transformations
            .iter()
            .map(|t| VariationDto {
                id: t.get_id().to_string(),
                name: t.get_name().to_string(),
                formula_latex: t.get_formula().to_string(),
            })
            .collect();

        GetAllVariationsCommandResult { variations }
    }
}
