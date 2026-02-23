use serde::Serialize;

pub struct GetAllVariationsCommand {}

#[derive(Serialize)]
pub struct VariationDto {
    pub id: String,
    pub name: String,
    pub formula_latex: String,
}

#[derive(Serialize)]
pub struct GetAllVariationsCommandResult {
    pub variations: Vec<VariationDto>,
}