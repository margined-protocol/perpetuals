mod tests;

use margined_utils::{
    create_entry_points_testing,
    testing::{NativeTokenScenario, SimpleScenario},
};
pub fn new_simple_scenario() -> SimpleScenario {
    SimpleScenario::new(
        Box::new(create_entry_points_testing!(crate)),
        Box::new(create_entry_points_testing!(cw20_base)),
        Box::new(
            create_entry_points_testing!(margined_engine)
                .with_reply(margined_engine::contract::reply),
        ),
        Box::new(create_entry_points_testing!(margined_vamm)),
        Box::new(create_entry_points_testing!(margined_insurance_fund)),
        Box::new(create_entry_points_testing!(mock_pricefeed)),
    )
}

pub fn new_native_token_scenario() -> NativeTokenScenario {
    NativeTokenScenario::new(
        Box::new(create_entry_points_testing!(crate)),
        Box::new(
            create_entry_points_testing!(margined_engine)
                .with_reply(margined_engine::contract::reply),
        ),
        Box::new(create_entry_points_testing!(margined_vamm)),
        Box::new(create_entry_points_testing!(margined_insurance_fund)),
        Box::new(create_entry_points_testing!(mock_pricefeed)),
    )
}
