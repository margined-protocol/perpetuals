use crate::{
    contracts::helpers::{
        EngineController, FeePoolController, InsuranceFundController, PricefeedController,
        VammController,
    },
    testing::to_decimals,
};
use cosmwasm_std::{Addr, Coin, Uint128};
use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg, MinterResponse};
use margined_common::asset::NATIVE_DENOM;
use margined_perp::margined_engine::{ExecuteMsg, InstantiateMsg};
use margined_perp::margined_fee_pool::InstantiateMsg as FeePoolInstantiateMsg;
use margined_perp::margined_insurance_fund::InstantiateMsg as InsuranceFundInstantiateMsg;
use margined_perp::margined_pricefeed::{
    ExecuteMsg as PricefeedExecuteMsg, InstantiateMsg as PricefeedInstantiateMsg,
};
use margined_perp::margined_vamm::{
    ExecuteMsg as VammExecuteMsg, InstantiateMsg as VammInstantiateMsg,
};
use osmosis_test_tube::{Module, OraichainTestApp, Wasm};
use test_tube::{cosmrs::proto::cosmwasm::wasm::v1::MsgExecuteContractResponse, Account};
use test_tube::{runner::Runner, SigningAccount};

static CW20_BYTES: &[u8] = include_bytes!("../../../../testdata/cw20-base.wasm");
static FEE_POOL_BYTES: &[u8] =
    include_bytes!("../../../../contracts/margined_fee_pool/artifacts/margined_fee_pool.wasm");
static ENGINE_BYTES: &[u8] =
    include_bytes!("../../../../contracts/margined_engine/artifacts/margined_engine.wasm");
static VAMM_BYTES: &[u8] =
    include_bytes!("../../../../contracts/margined_vamm/artifacts/margined_vamm.wasm");
static INSURANCE_BYTES: &[u8] = include_bytes!(
    "../../../../contracts/margined_insurance_fund/artifacts/margined_insurance_fund.wasm"
);
static PRICE_FEED_BYTES: &[u8] =
    include_bytes!("../../../../contracts/margined_pricefeed/artifacts/margined_pricefeed.wasm");

pub struct TestTubeScenario {
    pub router: OraichainTestApp,
    pub accounts: Vec<SigningAccount>,
    pub fee_pool: FeePoolController,
    pub usdc: Cw20Contract,
    pub vamm: VammController,
    pub engine: EngineController,
    pub pricefeed: PricefeedController,
    pub insurance_fund: InsuranceFundController,
}

impl Default for TestTubeScenario {
    fn default() -> Self {
        Self::new(None, None, None, None, None, None)
    }
}

impl TestTubeScenario {
    pub fn new(
        fee_pool_code: Option<&[u8]>,
        cw20_code: Option<&[u8]>,
        engine_code: Option<&[u8]>,
        vamm_code: Option<&[u8]>,
        insurance_fund_code: Option<&[u8]>,
        pricefeed_code: Option<&[u8]>,
    ) -> Self {
        let router = OraichainTestApp::default();

        let init_funds = [Coin::new(5_000_000_000_000u128, NATIVE_DENOM)];

        let accounts = router.init_accounts(&init_funds, 5).unwrap();
        // let owner = &accounts[0];

        let (owner, alice, bob, david) = (&accounts[0], &accounts[1], &accounts[2], &accounts[4]);

        let wasm = Wasm::new(&router);
        let fee_pool_id = wasm
            .store_code(fee_pool_code.unwrap_or(FEE_POOL_BYTES), None, owner)
            .unwrap()
            .data
            .code_id;

        let usdc_id = wasm
            .store_code(cw20_code.unwrap_or(CW20_BYTES), None, owner)
            .unwrap()
            .data
            .code_id;

        let engine_id = wasm
            .store_code(engine_code.unwrap_or(ENGINE_BYTES), None, owner)
            .unwrap()
            .data
            .code_id;

        let vamm_id = wasm
            .store_code(vamm_code.unwrap_or(VAMM_BYTES), None, owner)
            .unwrap()
            .data
            .code_id;
        let insurance_fund_id = wasm
            .store_code(insurance_fund_code.unwrap_or(INSURANCE_BYTES), None, owner)
            .unwrap()
            .data
            .code_id;
        let pricefeed_id = wasm
            .store_code(pricefeed_code.unwrap_or(PRICE_FEED_BYTES), None, owner)
            .unwrap()
            .data
            .code_id;

        let fee_pool_addr = wasm
            .instantiate(
                fee_pool_id,
                &FeePoolInstantiateMsg {},
                Some(&owner.address()),
                Some("fee_pool"),
                &[],
                owner,
            )
            .unwrap()
            .data
            .address;
        let fee_pool = FeePoolController(Addr::unchecked(fee_pool_addr));

        let usdc_addr = wasm
            .instantiate(
                usdc_id,
                &cw20_base::msg::InstantiateMsg {
                    name: "USDC".to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 9, //see here
                    initial_balances: vec![
                        Cw20Coin {
                            address: alice.address(),
                            amount: to_decimals(5000),
                        },
                        Cw20Coin {
                            address: bob.address(),
                            amount: to_decimals(5000),
                        },
                        Cw20Coin {
                            address: david.address(),
                            amount: to_decimals(5000),
                        },
                    ],
                    mint: Some(MinterResponse {
                        minter: owner.address(),
                        cap: None,
                    }),
                    marketing: None,
                },
                Some(&owner.address()),
                Some("cw20"),
                &[],
                owner,
            )
            .unwrap()
            .data
            .address;

        let usdc = Cw20Contract(Addr::unchecked(usdc_addr.clone()));

        // set up margined engine contract
        let engine_addr = wasm
            .instantiate(
                engine_id,
                &InstantiateMsg {
                    pauser: owner.address(),
                    operator: None,
                    insurance_fund: None,
                    fee_pool: fee_pool.0.to_string(),
                    eligible_collateral: usdc.0.to_string(),
                    initial_margin_ratio: Uint128::from(50_000_000u128), // 0.05
                    maintenance_margin_ratio: Uint128::from(50_000_000u128), // 0.05
                    tp_sl_spread: Uint128::from(50_000_000u128),         // 0.05
                    liquidation_fee: Uint128::from(50_000_000u128),      // 0.05
                },
                Some(&owner.address()),
                Some("engine"),
                &[],
                owner,
            )
            .unwrap()
            .data
            .address;
        let engine = EngineController(Addr::unchecked(engine_addr.clone()));

        let insurance_fund_addr = wasm
            .instantiate(
                insurance_fund_id,
                &InsuranceFundInstantiateMsg {
                    engine: engine.0.to_string(),
                },
                Some(&owner.address()),
                Some("insurance_fund"),
                &[],
                owner,
            )
            .unwrap()
            .data
            .address;
        let insurance_fund = InsuranceFundController(Addr::unchecked(insurance_fund_addr.clone()));

        // set insurance fund in margin engine
        wasm.execute(
            &engine_addr,
            &ExecuteMsg::UpdateConfig {
                owner: None,
                insurance_fund: Some(insurance_fund.0.to_string()),
                fee_pool: None,
                initial_margin_ratio: None,
                maintenance_margin_ratio: None,
                partial_liquidation_ratio: None,
                tp_sl_spread: None,
                liquidation_fee: None,
            },
            &[],
            owner,
        )
        .unwrap();

        wasm.execute(
            &usdc_addr,
            &Cw20ExecuteMsg::Mint {
                recipient: insurance_fund_addr.to_string(),
                amount: to_decimals(5000),
            },
            &[],
            owner,
        )
        .unwrap();

        let pricefeed_addr = wasm
            .instantiate(
                pricefeed_id,
                &PricefeedInstantiateMsg {
                    oracle_hub_contract: "oracle_hub0000".to_string(),
                },
                Some(&owner.address()),
                Some("pricefeed"),
                &[],
                owner,
            )
            .unwrap()
            .data
            .address;
        let pricefeed = PricefeedController(Addr::unchecked(pricefeed_addr.clone()));

        let vamm_addr = wasm
            .instantiate(
                vamm_id,
                &VammInstantiateMsg {
                    decimals: 9u8, //see here
                    quote_asset: "USD".to_string(),
                    base_asset: "ETH".to_string(),
                    quote_asset_reserve: to_decimals(1_000),
                    base_asset_reserve: to_decimals(100),
                    funding_period: 86_400_u64, // funding period is 1 day to make calcs easier
                    toll_ratio: Uint128::zero(),
                    spread_ratio: Uint128::zero(),
                    fluctuation_limit_ratio: Uint128::zero(),
                    pricefeed: pricefeed_addr.to_string(),
                    margin_engine: None,
                    insurance_fund: Some(insurance_fund_addr.to_string()),
                    initial_margin_ratio: Uint128::from(50_000u128),
                },
                Some(&owner.address()),
                Some("vamm"),
                &[],
                owner,
            )
            .unwrap()
            .data
            .address;
        let vamm = VammController(Addr::unchecked(vamm_addr.clone()));

        // set margin engine in vamm
        wasm.execute(
            &vamm_addr,
            &VammExecuteMsg::UpdateConfig {
                base_asset_holding_cap: None,
                open_interest_notional_cap: None,
                toll_ratio: None,
                spread_ratio: None,
                fluctuation_limit_ratio: None,
                margin_engine: Some(engine_addr.to_string()),
                insurance_fund: None,
                pricefeed: None,
                spot_price_twap_interval: None,
                initial_margin_ratio: None,
            },
            &[],
            owner,
        )
        .unwrap();

        // set open and register
        let msg = vamm.set_open(true).unwrap();
        router
            .execute_cosmos_msgs::<MsgExecuteContractResponse>(&[msg], owner)
            .unwrap();

        let msg = insurance_fund.add_vamm(vamm.0.to_string()).unwrap();
        router
            .execute_cosmos_msgs::<MsgExecuteContractResponse>(&[msg], owner)
            .unwrap();

        // create allowance for alice
        wasm.execute(
            &usdc_addr,
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: engine_addr.to_string(),
                amount: to_decimals(2000),
                expires: None,
            },
            &[],
            alice,
        )
        .unwrap();

        // create allowance for bob
        wasm.execute(
            &usdc_addr,
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: engine_addr.to_string(),
                amount: to_decimals(2000),
                expires: None,
            },
            &[],
            bob,
        )
        .unwrap();

        // create allowance for david
        wasm.execute(
            &usdc_addr,
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: engine_addr.to_string(),
                amount: to_decimals(100),
                expires: None,
            },
            &[],
            david,
        )
        .unwrap();

        // append a price to the mock pricefeed
        wasm.execute(
            &pricefeed_addr,
            &PricefeedExecuteMsg::AppendPrice {
                key: "ETH".to_string(),
                price: Uint128::from(10_000_000_000u128),
                timestamp: 1_000_000_000u64,
            },
            &[],
            owner,
        )
        .unwrap();

        Self {
            router,
            accounts,
            fee_pool,
            usdc,
            pricefeed,
            vamm,
            engine,
            insurance_fund,
        }
    }
}
