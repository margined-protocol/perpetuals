mod bad_debt_tests;
mod cw_token_add_remove_margin_tests;
mod cw_token_liquidation_frontrun_hack_tests;
mod cw_token_liquidation_tests;
mod cw_token_pay_funding_tests;
mod cw_token_position_fee_tests;
mod fee_calculation_tests;
mod fluctuation_tests;
mod margin_engine_tests;
mod margin_ratio_tests;
mod native_token_add_remove_margin_tests;
mod native_token_liquidation_frontrun_hack_tests;
mod native_token_liquidation_tests;
mod native_token_pay_funding_tests;
mod native_token_position_fee_tests;
mod open_interest_notional_tests;
mod pausable_tests;
mod personal_position_tests;
mod position_liquidation_tests;
mod position_tests;
mod position_upper_bound_tests;
mod tests;
mod tp_sl_test;
mod whitelist_tests;

mod gas_integration_tests;

mod vamm_leverage_test;

use margined_utils::{
    create_entry_points_testing,
    testing::{NativeTokenScenario, SimpleScenario},
};
pub fn new_simple_scenario() -> SimpleScenario {
    SimpleScenario::new(
        Box::new(create_entry_points_testing!(margined_fee_pool)),
        Box::new(create_entry_points_testing!(cw20_base)),
        Box::new(create_entry_points_testing!(crate).with_reply(crate::contract::reply)),
        Box::new(create_entry_points_testing!(margined_vamm)),
        Box::new(create_entry_points_testing!(margined_insurance_fund)),
        Box::new(create_entry_points_testing!(mock_pricefeed)),
    )
}

pub fn new_native_token_scenario() -> NativeTokenScenario {
    NativeTokenScenario::new(
        Box::new(create_entry_points_testing!(margined_fee_pool)),
        Box::new(create_entry_points_testing!(crate).with_reply(crate::contract::reply)),
        Box::new(create_entry_points_testing!(margined_vamm)),
        Box::new(create_entry_points_testing!(margined_insurance_fund)),
        Box::new(create_entry_points_testing!(mock_pricefeed)),
    )
}
