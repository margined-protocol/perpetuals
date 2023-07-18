use crate::contracts::helpers::{
    margined_engine::EngineController, margined_fee_pool::FeePoolController,
    margined_insurance_fund::InsuranceFundController, margined_pricefeed::PricefeedController,
    margined_vamm::VammController,
};
use crate::tools::fund_calculator::calculate_funds_needed;
use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, Empty, Response, Uint128};
use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg, MinterResponse};
use margined_common::asset::NATIVE_DENOM;
use margined_perp::margined_engine::{ExecuteMsg, InstantiateMsg, Side};
use margined_perp::margined_fee_pool::InstantiateMsg as FeePoolInstantiateMsg;
use margined_perp::margined_insurance_fund::InstantiateMsg as InsuranceFundInstantiateMsg;
use margined_perp::margined_pricefeed::{
    ExecuteMsg as PricefeedExecuteMsg, InstantiateMsg as PricefeedInstantiateMsg,
};
use margined_perp::margined_vamm::{
    ExecuteMsg as VammExecuteMsg, InstantiateMsg as VammInstantiateMsg,
};
// use terra_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use cw_multi_test::{App, AppBuilder, Contract, Executor};

pub type ContractCode = Box<dyn Contract<Empty>>;

pub struct ContractInfo {
    pub addr: Addr,
    pub id: u64,
}

pub struct NativeTokenScenario {
    pub router: App,
    pub owner: Addr,
    pub alice: Addr,
    pub bank: Addr,
    pub bob: Addr,
    pub carol: Addr,
    pub david: Addr,
    pub fee_pool: FeePoolController,
    pub vamm: VammController,
    pub engine: EngineController,
    pub pricefeed: PricefeedController,
    pub insurance_fund: InsuranceFundController,
}

impl NativeTokenScenario {
    pub fn new(
        fee_pool_code: ContractCode,
        engine_code: ContractCode,
        vamm_code: ContractCode,
        insurance_fund_code: ContractCode,
        pricefeed_code: ContractCode,
    ) -> Self {
        let bank = Addr::unchecked("bank"); //just holds a lot of funds to send to people
        let owner = Addr::unchecked("owner");
        let alice = Addr::unchecked("alice");
        let bob = Addr::unchecked("bob");
        let carol = Addr::unchecked("carol");
        let david = Addr::unchecked("david");

        let init_funds = vec![Coin::new(5_000u128 * 10u128.pow(6), NATIVE_DENOM)];
        let bank_funds = vec![
            Coin::new(100_000u128 * 10u128.pow(6), NATIVE_DENOM),
            Coin::new(100_000u128 * 10u128.pow(6), "ucosmos"),
        ];

        let mut router: App = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &alice, init_funds.clone())
                .unwrap();
            router
                .bank
                .init_balance(storage, &bob, init_funds.clone())
                .unwrap();
            router
                .bank
                .init_balance(storage, &david, init_funds.clone())
                .unwrap();
            router
                .bank
                .init_balance(storage, &bank, bank_funds.clone())
                .unwrap();
        });

        let fee_pool_id = router.store_code(fee_pool_code);
        let engine_id = router.store_code(engine_code);
        let vamm_id = router.store_code(vamm_code);
        let insurance_fund_id = router.store_code(insurance_fund_code);
        let pricefeed_id = router.store_code(pricefeed_code);

        let fee_pool_addr = router
            .instantiate_contract(
                fee_pool_id,
                owner.clone(),
                &FeePoolInstantiateMsg {},
                &[],
                "fee_pool",
                None,
            )
            .unwrap();
        let fee_pool = FeePoolController(fee_pool_addr);

        // set up margined engine contract
        let engine_addr = router
            .instantiate_contract(
                engine_id,
                owner.clone(),
                &InstantiateMsg {
                    pauser: owner.to_string(),
                    insurance_fund: None,
                    fee_pool: fee_pool.0.to_string(),
                    eligible_collateral: NATIVE_DENOM.to_string(),
                    initial_margin_ratio: Uint128::from(50_000u128), // 0.05
                    maintenance_margin_ratio: Uint128::from(50_000u128), // 0.05
                    liquidation_fee: Uint128::from(50_000u128),      // 0.05
                },
                &[],
                "engine",
                None,
            )
            .unwrap();
        let engine = EngineController(engine_addr.clone());

        let insurance_fund_addr = router
            .instantiate_contract(
                insurance_fund_id,
                owner.clone(),
                &InsuranceFundInstantiateMsg {
                    engine: engine_addr.to_string(),
                },
                &[],
                "insurance_fund",
                None,
            )
            .unwrap();
        let insurance_fund = InsuranceFundController(insurance_fund_addr);

        // send insurance fund funds
        let msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: insurance_fund.0.to_string(),
            amount: init_funds,
        });
        router.execute(bank.clone(), msg).unwrap();

        // set insurance fund in margin engine
        router
            .execute_contract(
                owner.clone(),
                engine_addr.clone(),
                &ExecuteMsg::UpdateConfig {
                    owner: None,
                    insurance_fund: Some(insurance_fund.0.to_string()),
                    fee_pool: None,
                    initial_margin_ratio: None,
                    maintenance_margin_ratio: None,
                    partial_liquidation_ratio: None,
                    liquidation_fee: None,
                },
                &[],
            )
            .unwrap();

        let pricefeed_addr = router
            .instantiate_contract(
                pricefeed_id,
                owner.clone(),
                &PricefeedInstantiateMsg {
                    oracle_hub_contract: "oracle_hub0000".to_string(),
                },
                &[],
                "pricefeed",
                None,
            )
            .unwrap();
        let pricefeed = PricefeedController(pricefeed_addr.clone());

        let vamm_addr = router
            .instantiate_contract(
                vamm_id,
                owner.clone(),
                &VammInstantiateMsg {
                    decimals: 6u8,
                    quote_asset: "USD".to_string(),
                    base_asset: "ETH".to_string(),
                    quote_asset_reserve: Uint128::from(1_000_000_000u128),
                    base_asset_reserve: Uint128::from(100_000_000u128),
                    funding_period: 86_400_u64, // funding period is 1 day to make calcs easier
                    toll_ratio: Uint128::zero(),
                    spread_ratio: Uint128::zero(),
                    fluctuation_limit_ratio: Uint128::zero(),
                    pricefeed: pricefeed_addr.to_string(),
                    margin_engine: None,
                    insurance_fund: Some(insurance_fund.0.to_string()),
                },
                &[],
                "vamm",
                None,
            )
            .unwrap();
        let vamm = VammController(vamm_addr.clone());

        // set margin engine in vamm
        router
            .execute_contract(
                owner.clone(),
                vamm_addr,
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
                },
                &[],
            )
            .unwrap();

        // set open and register
        let msg = vamm.set_open(true).unwrap();
        router.execute(owner.clone(), msg).unwrap();

        let msg = insurance_fund.add_vamm(vamm.0.to_string()).unwrap();
        router.execute(owner.clone(), msg).unwrap();

        // append a price to the mock pricefeed
        router
            .execute_contract(
                owner.clone(),
                pricefeed_addr,
                &PricefeedExecuteMsg::AppendPrice {
                    key: "ETH".to_string(),
                    price: Uint128::from(10_000_000u128),
                    timestamp: 1_000_000_000u64,
                },
                &[],
            )
            .unwrap();

        Self {
            router,
            owner,
            alice,
            bob,
            bank,
            carol,
            david,
            fee_pool,
            pricefeed,
            vamm,
            engine,
            insurance_fund,
        }
    }

    pub fn open_small_position(
        &mut self,
        account: Addr,
        side: Side,
        quote_asset_amount: Uint128,
        leverage: Uint128,
        count: u64,
    ) {
        let funds = calculate_funds_needed(
            &self.router.wrap(),
            quote_asset_amount,
            leverage,
            self.vamm.addr()
        ).unwrap();
        for _ in 0..count {
            let msg = self
                .engine
                .open_position(
                    self.vamm.0.to_string(),
                    side.clone(),
                    quote_asset_amount,
                    leverage,
                    Uint128::zero(),
                    Some(Uint128::zero()),
                    Uint128::zero(),
                    funds.clone(),
                )
                .unwrap();
            self.router.execute(account.clone(), msg).unwrap();

            self.router.update_block(|block| {
                block.time = block.time.plus_seconds(15);
                block.height += 1;
            });
        }
    }
}

pub struct SimpleScenario {
    pub router: App,
    pub owner: Addr,
    pub alice: Addr,
    pub bob: Addr,
    pub carol: Addr,
    pub david: Addr,
    pub fee_pool: FeePoolController,
    pub usdc: Cw20Contract,
    pub vamm: VammController,
    pub engine: EngineController,
    pub pricefeed: PricefeedController,
    pub insurance_fund: InsuranceFundController,
}

impl SimpleScenario {
    pub fn new(
        fee_pool_code: ContractCode,
        cw20_code: ContractCode,
        engine_code: ContractCode,
        vamm_code: ContractCode,
        insurance_fund_code: ContractCode,
        pricefeed_code: ContractCode,
    ) -> Self {
        let mut router = AppBuilder::new().build(|_router, _, _storage| {});

        let owner = Addr::unchecked("owner");
        let alice = Addr::unchecked("alice");
        let bob = Addr::unchecked("bob");
        let carol = Addr::unchecked("carol");
        let david = Addr::unchecked("david");

        let fee_pool_id = router.store_code(fee_pool_code);
        let usdc_id = router.store_code(cw20_code);
        let engine_id = router.store_code(engine_code);
        let vamm_id = router.store_code(vamm_code);
        let insurance_fund_id = router.store_code(insurance_fund_code);
        let pricefeed_id = router.store_code(pricefeed_code);

        let fee_pool_addr = router
            .instantiate_contract(
                fee_pool_id,
                owner.clone(),
                &FeePoolInstantiateMsg {},
                &[],
                "fee_pool",
                None,
            )
            .unwrap();
        let fee_pool = FeePoolController(fee_pool_addr);

        let usdc_addr = router
            .instantiate_contract(
                usdc_id,
                owner.clone(),
                &cw20_base::msg::InstantiateMsg {
                    name: "USDC".to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 9, //see here
                    initial_balances: vec![
                        Cw20Coin {
                            address: alice.to_string(),
                            amount: to_decimals(5000),
                        },
                        Cw20Coin {
                            address: bob.to_string(),
                            amount: to_decimals(5000),
                        },
                        Cw20Coin {
                            address: david.to_string(),
                            amount: to_decimals(5000),
                        },
                    ],
                    mint: Some(MinterResponse {
                        minter: owner.to_string(),
                        cap: None,
                    }),
                    marketing: None,
                },
                &[],
                "cw20",
                None,
            )
            .unwrap();

        let usdc = Cw20Contract(usdc_addr.clone());

        // set up margined engine contract
        let engine_addr = router
            .instantiate_contract(
                engine_id,
                owner.clone(),
                &InstantiateMsg {
                    pauser: owner.to_string(),
                    insurance_fund: None,
                    fee_pool: fee_pool.0.to_string(),
                    eligible_collateral: usdc.0.to_string(),
                    initial_margin_ratio: Uint128::from(50_000_000u128), // 0.05
                    maintenance_margin_ratio: Uint128::from(50_000_000u128), // 0.05
                    liquidation_fee: Uint128::from(50_000_000u128),      // 0.05
                },
                &[],
                "engine",
                None,
            )
            .unwrap();
        let engine = EngineController(engine_addr.clone());

        let insurance_fund_addr = router
            .instantiate_contract(
                insurance_fund_id,
                owner.clone(),
                &InsuranceFundInstantiateMsg {
                    engine: engine.0.to_string(),
                },
                &[],
                "insurance_fund",
                None,
            )
            .unwrap();
        let insurance_fund = InsuranceFundController(insurance_fund_addr.clone());

        // set insurance fund in margin engine
        router
            .execute_contract(
                owner.clone(),
                engine.0.clone(),
                &ExecuteMsg::UpdateConfig {
                    owner: None,
                    insurance_fund: Some(insurance_fund.0.to_string()),
                    fee_pool: None,
                    initial_margin_ratio: None,
                    maintenance_margin_ratio: None,
                    partial_liquidation_ratio: None,
                    liquidation_fee: None,
                },
                &[],
            )
            .unwrap();

        router
            .execute_contract(
                owner.clone(),
                usdc_addr.clone(),
                &Cw20ExecuteMsg::Mint {
                    recipient: insurance_fund_addr.to_string(),
                    amount: to_decimals(5000),
                },
                &[],
            )
            .unwrap();

        let pricefeed_addr = router
            .instantiate_contract(
                pricefeed_id,
                owner.clone(),
                &PricefeedInstantiateMsg {
                    oracle_hub_contract: "oracle_hub0000".to_string(),
                },
                &[],
                "pricefeed",
                None,
            )
            .unwrap();
        let pricefeed = PricefeedController(pricefeed_addr.clone());

        let vamm_addr = router
            .instantiate_contract(
                vamm_id,
                owner.clone(),
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
                },
                &[],
                "vamm",
                None,
            )
            .unwrap();
        let vamm = VammController(vamm_addr.clone());

        // set margin engine in vamm
        router
            .execute_contract(
                owner.clone(),
                vamm_addr,
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
                },
                &[],
            )
            .unwrap();

        // set open and register
        let msg = vamm.set_open(true).unwrap();
        router.execute(owner.clone(), msg).unwrap();

        let msg = insurance_fund.add_vamm(vamm.0.to_string()).unwrap();
        router.execute(owner.clone(), msg).unwrap();

        // create allowance for alice
        router
            .execute_contract(
                alice.clone(),
                usdc_addr.clone(),
                &Cw20ExecuteMsg::IncreaseAllowance {
                    spender: engine_addr.to_string(),
                    amount: to_decimals(2000),
                    expires: None,
                },
                &[],
            )
            .unwrap();

        // create allowance for bob
        router
            .execute_contract(
                bob.clone(),
                usdc_addr.clone(),
                &Cw20ExecuteMsg::IncreaseAllowance {
                    spender: engine_addr.to_string(),
                    amount: to_decimals(2000),
                    expires: None,
                },
                &[],
            )
            .unwrap();

        // create allowance for david
        router
            .execute_contract(
                david.clone(),
                usdc_addr,
                &Cw20ExecuteMsg::IncreaseAllowance {
                    spender: engine_addr.to_string(),
                    amount: to_decimals(100),
                    expires: None,
                },
                &[],
            )
            .unwrap();

        // append a price to the mock pricefeed
        router
            .execute_contract(
                owner.clone(),
                pricefeed_addr,
                &PricefeedExecuteMsg::AppendPrice {
                    key: "ETH".to_string(),
                    price: Uint128::from(10_000_000_000u128),
                    timestamp: 1_000_000_000u64,
                },
                &[],
            )
            .unwrap();

        Self {
            router,
            owner,
            alice,
            bob,
            carol,
            david,
            fee_pool,
            usdc,
            pricefeed,
            vamm,
            engine,
            insurance_fund,
        }
    }

    pub fn open_small_position(
        &mut self,
        account: Addr,
        side: Side,
        quote_asset_amount: Uint128,
        leverage: Uint128,
        count: u64,
    ) {
        for _ in 0..count {
            let msg = self
                .engine
                .open_position(
                    self.vamm.0.to_string(),
                    side.clone(),
                    quote_asset_amount,
                    leverage,
                    Uint128::zero(),
                    Some(Uint128::zero()),
                    Uint128::zero(),
                    vec![],
                )
                .unwrap();
            self.router.execute(account.clone(), msg).unwrap();

            self.router.update_block(|block| {
                block.time = block.time.plus_seconds(15);
                block.height += 1;
            });
        }
    }
}

pub struct VammScenario {
    pub router: App,
    pub owner: Addr,
    pub alice: Addr,
    pub bob: Addr,
    pub carol: Addr,
    pub usdc: Cw20Contract,
    pub vamm: VammController,
    pub pricefeed: PricefeedController,
}

impl VammScenario {
    pub fn new(
        cw20_code: ContractCode,
        vamm_code: ContractCode,
        pricefeed_code: ContractCode,
    ) -> Self {
        let mut router = AppBuilder::new().build(|_router, _, _storage| {});

        let owner = Addr::unchecked("owner");
        let alice = Addr::unchecked("alice");
        let bob = Addr::unchecked("bob");
        let carol = Addr::unchecked("carol");

        let usdc_id = router.store_code(cw20_code);
        let vamm_id = router.store_code(vamm_code);
        let pricefeed_id = router.store_code(pricefeed_code);

        let usdc_addr = router
            .instantiate_contract(
                usdc_id,
                owner.clone(),
                &cw20_base::msg::InstantiateMsg {
                    name: "USDC".to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 9, //see here
                    initial_balances: vec![
                        Cw20Coin {
                            address: alice.to_string(),
                            amount: to_decimals(5000),
                        },
                        Cw20Coin {
                            address: bob.to_string(),
                            amount: to_decimals(5000),
                        },
                        Cw20Coin {
                            address: carol.to_string(),
                            amount: to_decimals(5000),
                        },
                    ],
                    mint: None,
                    marketing: None,
                },
                &[],
                "cw20",
                None,
            )
            .unwrap();

        let usdc = Cw20Contract(usdc_addr);

        let pricefeed_addr = router
            .instantiate_contract(
                pricefeed_id,
                owner.clone(),
                &PricefeedInstantiateMsg {
                    oracle_hub_contract: "oracle_hub0000".to_string(),
                },
                &[],
                "pricefeed",
                None,
            )
            .unwrap();
        let pricefeed = PricefeedController(pricefeed_addr.clone());

        let vamm_addr = router
            .instantiate_contract(
                vamm_id,
                owner.clone(),
                &VammInstantiateMsg {
                    decimals: 9u8, //see here
                    quote_asset: "USD".to_string(),
                    base_asset: "ETH".to_string(),
                    quote_asset_reserve: to_decimals(1_000),
                    base_asset_reserve: to_decimals(100),
                    funding_period: 3_600_u64, // funding period is 1 day to make calcs easier
                    toll_ratio: Uint128::from(10_000_000u128), // 0.01
                    spread_ratio: Uint128::from(10_000_000u128), // 0.01
                    fluctuation_limit_ratio: Uint128::from(10_000_000u128), // 0.01
                    pricefeed: pricefeed_addr.to_string(),
                    margin_engine: Some(owner.to_string()),
                    insurance_fund: Some("insurance_fund".to_string()),
                },
                &[],
                "vamm",
                None,
            )
            .unwrap();
        let vamm = VammController(vamm_addr);

        let msg = vamm.set_open(true).unwrap();
        router.execute(owner.clone(), msg).unwrap();

        Self {
            router,
            owner,
            alice,
            bob,
            carol,
            usdc,
            pricefeed,
            vamm,
        }
    }
}

pub struct ShutdownScenario {
    pub router: App,
    pub owner: Addr,
    pub vamm1: VammController,
    pub vamm2: VammController,
    pub vamm3: VammController,
    pub vamm4: VammController,
    pub vamm5: VammController,
    pub insurance_fund: InsuranceFundController,
    pub pricefeed: PricefeedController,
}

impl ShutdownScenario {
    pub fn new(
        insurance_fund_code: ContractCode,
        engine_code: ContractCode,
        vamm_code: ContractCode,

        pricefeed_code: ContractCode,
    ) -> Self {
        let mut router = AppBuilder::new().build(|_router, _, _storage| {});

        let owner = Addr::unchecked("owner");

        let insurance_fund_id = router.store_code(insurance_fund_code);
        let engine_id = router.store_code(engine_code);
        let vamm_id = router.store_code(vamm_code);
        let pricefeed_id = router.store_code(pricefeed_code);

        // set up margined engine contract
        let engine_addr = router
            .instantiate_contract(
                engine_id,
                owner.clone(),
                &InstantiateMsg {
                    pauser: owner.to_string(),
                    insurance_fund: None,
                    fee_pool: "fee_pool".to_string(),
                    eligible_collateral: NATIVE_DENOM.to_string(),
                    initial_margin_ratio: Uint128::from(50_000u128), // 0.05
                    maintenance_margin_ratio: Uint128::from(50_000u128), // 0.05
                    liquidation_fee: Uint128::from(50_000u128),      // 0.05
                },
                &[],
                "engine",
                None,
            )
            .unwrap();
        let engine = EngineController(engine_addr);

        let insurance_fund_addr = router
            .instantiate_contract(
                insurance_fund_id,
                owner.clone(),
                &InsuranceFundInstantiateMsg {
                    engine: engine.0.to_string(),
                },
                &[],
                "insurance_fund",
                None,
            )
            .unwrap();
        let insurance_fund = InsuranceFundController(insurance_fund_addr.clone());

        let pricefeed_addr = router
            .instantiate_contract(
                pricefeed_id,
                owner.clone(),
                &PricefeedInstantiateMsg {
                    oracle_hub_contract: "oracle_hub0000".to_string(),
                },
                &[],
                "pricefeed",
                None,
            )
            .unwrap();
        let pricefeed = PricefeedController(pricefeed_addr.clone());

        let vamm1_addr = router
            .instantiate_contract(
                vamm_id,
                insurance_fund_addr.clone(),
                &VammInstantiateMsg {
                    decimals: 6u8, //see here
                    quote_asset: "USD".to_string(),
                    base_asset: "ETH".to_string(),
                    quote_asset_reserve: Uint128::from(1_000_000_000u128),
                    base_asset_reserve: Uint128::from(100_000_000u128),
                    funding_period: 3_600_u64, // funding period is 1 day to make calcs easier
                    toll_ratio: Uint128::from(10_000u128), // 0.01
                    spread_ratio: Uint128::from(10_000u128), // 0.01
                    fluctuation_limit_ratio: Uint128::from(10_000u128), // 0.01
                    pricefeed: pricefeed_addr.to_string(),
                    margin_engine: Some(owner.to_string()),
                    insurance_fund: Some(insurance_fund_addr.to_string()),
                },
                &[],
                "vamm1",
                None,
            )
            .unwrap();
        let vamm1 = VammController(vamm1_addr);

        let msg = vamm1.set_open(true).unwrap();
        router.execute(insurance_fund_addr.clone(), msg).unwrap();

        let vamm2_addr = router
            .instantiate_contract(
                vamm_id,
                insurance_fund_addr.clone(),
                &VammInstantiateMsg {
                    decimals: 6u8, //see here
                    quote_asset: "USD".to_string(),
                    base_asset: "ETH".to_string(),
                    quote_asset_reserve: Uint128::from(1_000_000_000u128),
                    base_asset_reserve: Uint128::from(100_000_000u128),
                    funding_period: 3_600_u64, // funding period is 1 day to make calcs easier
                    toll_ratio: Uint128::from(10_000u128), // 0.01
                    spread_ratio: Uint128::from(10_000u128), // 0.01
                    fluctuation_limit_ratio: Uint128::from(10_000u128), // 0.01
                    pricefeed: pricefeed_addr.to_string(),
                    margin_engine: Some(owner.to_string()),
                    insurance_fund: Some(insurance_fund_addr.to_string()),
                },
                &[],
                "vamm2",
                None,
            )
            .unwrap();
        let vamm2 = VammController(vamm2_addr);

        let msg = vamm2.set_open(true).unwrap();
        router.execute(insurance_fund_addr.clone(), msg).unwrap();

        let vamm3_addr = router
            .instantiate_contract(
                vamm_id,
                insurance_fund_addr.clone(),
                &VammInstantiateMsg {
                    decimals: 6u8, //see here
                    quote_asset: "USD".to_string(),
                    base_asset: "ETH".to_string(),
                    quote_asset_reserve: Uint128::from(1_000_000_000u128),
                    base_asset_reserve: Uint128::from(100_000_000u128),
                    funding_period: 3_600_u64, // funding period is 1 day to make calcs easier
                    toll_ratio: Uint128::from(10_000u128), // 0.01
                    spread_ratio: Uint128::from(10_000u128), // 0.01
                    fluctuation_limit_ratio: Uint128::from(10_000u128), // 0.01
                    pricefeed: pricefeed_addr.to_string(),
                    margin_engine: Some(owner.to_string()),
                    insurance_fund: Some(insurance_fund_addr.to_string()),
                },
                &[],
                "vamm3",
                None,
            )
            .unwrap();
        let vamm3 = VammController(vamm3_addr);

        let msg = vamm3.set_open(true).unwrap();
        router.execute(insurance_fund_addr.clone(), msg).unwrap();

        let vamm4_addr = router
            .instantiate_contract(
                vamm_id,
                insurance_fund_addr.clone(),
                &VammInstantiateMsg {
                    decimals: 6u8, //see here
                    quote_asset: "USD".to_string(),
                    base_asset: "ETH".to_string(),
                    quote_asset_reserve: Uint128::from(1_000_000_000u128),
                    base_asset_reserve: Uint128::from(100_000_000u128),
                    funding_period: 3_600_u64, // funding period is 1 day to make calcs easier
                    toll_ratio: Uint128::from(10_000u128), // 0.01
                    spread_ratio: Uint128::from(10_000u128), // 0.01
                    fluctuation_limit_ratio: Uint128::from(10_000u128), // 0.01
                    pricefeed: pricefeed_addr.to_string(),
                    margin_engine: Some(owner.to_string()),
                    insurance_fund: Some(insurance_fund.0.to_string()),
                },
                &[],
                "vamm4",
                None,
            )
            .unwrap();
        let vamm4 = VammController(vamm4_addr);

        let msg = vamm4.set_open(true).unwrap();
        router.execute(insurance_fund_addr, msg).unwrap();

        let vamm5_addr = router
            .instantiate_contract(
                vamm_id,
                insurance_fund.0.clone(),
                &VammInstantiateMsg {
                    decimals: 7u8, //see here
                    quote_asset: "USD".to_string(),
                    base_asset: "ETH".to_string(),
                    quote_asset_reserve: Uint128::from(1_000_000_000u128),
                    base_asset_reserve: Uint128::from(100_000_000u128),
                    funding_period: 3_600_u64, // funding period is 1 day to make calcs easier
                    toll_ratio: Uint128::from(10_000u128), // 0.01
                    spread_ratio: Uint128::from(10_000u128), // 0.01
                    fluctuation_limit_ratio: Uint128::from(10_000u128), // 0.01
                    pricefeed: pricefeed_addr.to_string(),
                    margin_engine: Some(owner.to_string()),
                    insurance_fund: Some(insurance_fund.0.to_string()),
                },
                &[],
                "vamm5",
                None,
            )
            .unwrap();
        let vamm5 = VammController(vamm5_addr);

        Self {
            router,
            owner,
            insurance_fund,
            vamm1,
            vamm2,
            vamm3,
            vamm4,
            vamm5,
            pricefeed,
        }
    }
}

pub const DECIMAL_MULTIPLIER: Uint128 = Uint128::new(1_000_000_000);

/// contract initialization
#[macro_export]
macro_rules! create_entry_points_testing {
    ($contract:ident) => {
        $crate::cw_multi_test::ContractWrapper::new_with_empty(
            $contract::contract::execute,
            $contract::contract::instantiate,
            $contract::contract::query,
        )
    };
}

// takes in a Uint128 and multiplies by the decimals just to make tests more legible
pub fn to_decimals(input: u64) -> Uint128 {
    Uint128::from(input) * DECIMAL_MULTIPLIER
}

pub fn parse_event(res: &Response, key: &str) -> String {
    let res = &res
        .attributes
        .iter()
        .find(|&attr| attr.key == key)
        .unwrap()
        .value;

    res.to_string()
}
