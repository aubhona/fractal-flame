use crate::app::use_cases::get_all_variations_command_handler::GetAllVariationsCommandHandler;
use crate::infra::Dependencies;

pub fn get_get_all_variations_command_handler(
    deps: &Dependencies,
) -> GetAllVariationsCommandHandler {
    GetAllVariationsCommandHandler::new(deps.transformations.clone())
}
