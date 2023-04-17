mod tests;

use margined_utils::{create_entry_points_testing, testing::ShutdownScenario};

pub fn new_shutdown_scenario() -> ShutdownScenario {
    ShutdownScenario::new(
        Box::new(create_entry_points_testing!(crate)),
        Box::new(
            create_entry_points_testing!(margined_engine)
                .with_reply(margined_engine::contract::reply),
        ),
        Box::new(create_entry_points_testing!(margined_vamm)),
        Box::new(create_entry_points_testing!(mock_pricefeed)),
    )
}
