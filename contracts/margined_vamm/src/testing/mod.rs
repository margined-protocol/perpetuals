mod fee_tests;
mod fluctuation_tests;
mod funding_tests;
mod get_price_tests;
mod set_open_tests;
mod spread_limit_tests;
mod swap_input_output_tests;
mod swap_tests;
mod twap_tests;

use margined_utils::{
    create_entry_points_testing,
    testing::{SimpleScenario, VammScenario},
};
pub fn new_simple_scenario() -> SimpleScenario {
    SimpleScenario::new(
        Box::new(create_entry_points_testing!(margined_fee_pool)),
        Box::new(create_entry_points_testing!(cw20_base)),
        Box::new(
            create_entry_points_testing!(margined_engine)
                .with_reply(margined_engine::contract::reply),
        ),
        Box::new(create_entry_points_testing!(crate)),
        Box::new(create_entry_points_testing!(margined_insurance_fund)),
        Box::new(create_entry_points_testing!(mock_pricefeed)),
    )
}

pub fn new_vammscenario() -> VammScenario {
    VammScenario::new(
        Box::new(create_entry_points_testing!(cw20_base)),
        Box::new(create_entry_points_testing!(crate)),
        Box::new(create_entry_points_testing!(mock_pricefeed)),
    )
}
