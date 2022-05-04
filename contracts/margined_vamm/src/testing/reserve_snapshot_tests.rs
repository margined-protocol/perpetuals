use crate::contract::{execute, instantiate, query};
use crate::state::ReserveSnapshot;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Timestamp, Uint128};
use margined_common::integer::Integer;
use margined_perp::margined_vamm::{
    Direction, ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse,
};
use margined_utils::scenarios::to_decimals;

#[test]
fn test_reserve_snapshot_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // open amm
    let info = mock_info("addr0000", &[]);
    execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SetOpen { open: true },
    )
    .unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::ReserveSnapshotHeight {},
    )
    .unwrap();
    let height: u64 = from_binary(&res).unwrap();
    assert_eq!(height, 1u64);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::ReserveSnapshots {
            start: None,
            limit: None,
        },
    )
    .unwrap();
    let snapshot: Vec<ReserveSnapshot> = from_binary(&res).unwrap();
    assert_eq!(
        snapshot,
        vec![ReserveSnapshot {
            quote_asset_reserve: Uint128::from(1_000_000_000_000u128),
            base_asset_reserve: Uint128::from(100_000_000_000u128),
            timestamp: Timestamp::from_nanos(1571797419879305533),
            block_height: 12345u64,
        }]
    );
}

#[test]
fn test_reserve_snapshot_limit() {
    let mut deps = mock_dependencies(&[]);
    let mut env = mock_env();
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // open amm
    let info = mock_info("addr0000", &[]);
    execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::SetOpen { open: true },
    )
    .unwrap();

    for n in 0..40 {
        // move to the next block
        env.block.time = env.block.time.plus_seconds(15u64);
        env.block.height += 1u64;

        let swap_msg = ExecuteMsg::SwapInput {
            direction: Direction::RemoveFromAmm,
            quote_asset_amount: to_decimals(100),
            base_asset_limit: Uint128::zero(),
            can_go_over_fluctuation: false,
        };

        let info = mock_info("addr0000", &[]);
        execute(deps.as_mut(), env.clone(), info, swap_msg).unwrap();

        // move to the next block
        env.block.time = env.block.time.plus_seconds(15u64);
        env.block.height += 1u64;

        let swap_msg = ExecuteMsg::SwapInput {
            direction: Direction::AddToAmm,
            quote_asset_amount: to_decimals(100),
            base_asset_limit: Uint128::zero(),
            can_go_over_fluctuation: false,
        };

        let info = mock_info("addr0000", &[]);
        execute(deps.as_mut(), env.clone(), info, swap_msg).unwrap();

        // check the height is correct
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ReserveSnapshotHeight {},
        )
        .unwrap();
        let height: u64 = from_binary(&res).unwrap();
        assert_eq!(height, (3 + (n * 2)));
    }

    let res = query(deps.as_ref(), env.clone(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            open: true,
            quote_asset_reserve: to_decimals(1_000),
            base_asset_reserve: Uint128::from(100_000_000_008u128),
            total_position_size: Integer::new_negative(8u128),
            funding_rate: Uint128::zero(),
            next_funding_time: 1_571_801_019u64,
        }
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::ReserveSnapshotHeight {},
    )
    .unwrap();
    let height: u64 = from_binary(&res).unwrap();
    assert_eq!(height, 81);

    // check that it gives default number back
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::ReserveSnapshots {
            start: None,
            limit: None,
        },
    )
    .unwrap();
    let snapshot: Vec<ReserveSnapshot> = from_binary(&res).unwrap();
    assert_eq!(snapshot.len(), 10usize);

    // check that it gives correct first snapshot
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::ReserveSnapshots {
            start: Some(0u64),
            limit: Some(1u32),
        },
    )
    .unwrap();
    let snapshot: Vec<ReserveSnapshot> = from_binary(&res).unwrap();
    assert_eq!(
        snapshot,
        vec![ReserveSnapshot {
            quote_asset_reserve: Uint128::from(1_000_000_000_000u128),
            base_asset_reserve: Uint128::from(100_000_000_000u128),
            timestamp: Timestamp::from_nanos(1571797419879305533),
            block_height: 12345u64,
        }]
    );
}
