use axum::{extract::State, Json};

use crate::app::use_cases::get_all_variations_command::{
    GetAllVariationsCommand, GetAllVariationsCommandResult,
};
use crate::app::use_cases::get_all_variations_command_handler::GetAllVariationsCommandHandler;
use crate::di;
use crate::infra::Dependencies;

pub async fn get_variations(
    State(deps): State<Dependencies>,
) -> Json<GetAllVariationsCommandResult> {
    let handler: GetAllVariationsCommandHandler = di::get_get_all_variations_command_handler(&deps);
    let result = handler.handle(GetAllVariationsCommand {});
    Json(result)
}
