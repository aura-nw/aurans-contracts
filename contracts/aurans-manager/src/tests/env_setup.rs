#[cfg(test)]
pub mod env {
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use crate::contract::{
        execute as ManagerExecute, instantiate as ManagerInstantiate, query as ManagerQuery,
    };

    use aurans_name::contract::{
        execute as NameExecute, instantiate as NameInstantiate, query as NameQuery,
    };

    use aurans_resolver::contract::{
        execute as ResolverExecute, instantiate as ResolverInstantiate, query as ResolverQuery,
    };

    use crate::msg::InstantiateMsg as ManagerInstantiateMsg;
    use aurans_name::msg::InstantiateMsg as NameInstantiateMsg;
    use aurans_resolver::msg::InstantiateMsg as ResolverInstantiateMsg;

    pub const ADMIN: &str = "aura1000000000000000000000000000000000admin";
    // pub const USER_1: &str = "aura1000000000000000000000000000000000user1";

    pub const NATIVE_DENOM: &str = "uaura";
    pub const NATIVE_BALANCE: u128 = 1_000_000_000_000u128;

    pub const NATIVE_DENOM_2: &str = "utaura";
    pub const NATIVE_BALANCE_2: u128 = 1_000_000_000_000u128;

    pub struct ContractInfo {
        pub contract_addr: String,
        pub contract_code_id: u64,
    }

    // create app instance and init balance of NATIVE token for admin
    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(ADMIN),
                    vec![
                        Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::new(NATIVE_BALANCE),
                        },
                        Coin {
                            denom: NATIVE_DENOM_2.to_string(),
                            amount: Uint128::new(NATIVE_BALANCE_2),
                        },
                    ],
                )
                .unwrap();
        })
    }

    // create aurans manager contract
    pub fn manager_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(ManagerExecute, ManagerInstantiate, ManagerQuery);
        Box::new(contract)
    }

    // create aurans name contract
    pub fn name_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(NameExecute, NameInstantiate, NameQuery);
        Box::new(contract)
    }

    // create aurans resolver contract
    pub fn resolver_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(ResolverExecute, ResolverInstantiate, ResolverQuery);
        Box::new(contract)
    }

    pub fn instantiate_contracts() -> (App, Vec<ContractInfo>) {
        // Create a new app instance
        let mut app = mock_app();
        // Create a vector to store all contract info ([halo factory - [0])
        let mut contract_info_vec: Vec<ContractInfo> = Vec::new();

        // store code of all contracts to the app and get the code ids
        let manager_contract_code_id = app.store_code(manager_contract_template());
        let name_contract_code_id = app.store_code(name_contract_template());
        let resolver_contract_code_id = app.store_code(resolver_contract_template());

        // instantiate aurans manager contract
        let manager_contract_addr = app
            .instantiate_contract(
                manager_contract_code_id,
                Addr::unchecked(ADMIN),
                &ManagerInstantiateMsg {
                    admin: ADMIN.to_string(),
                },
                &[],
                "test instantiate contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: manager_contract_addr.to_string(),
            contract_code_id: manager_contract_code_id,
        });

        // instantiate aurans name contract
        let name_contract_addr = app
            .instantiate_contract(
                name_contract_code_id,
                Addr::unchecked(ADMIN),
                &NameInstantiateMsg {
                    admin: ADMIN.to_string(),
                },
                &[],
                "test instantiate contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: name_contract_addr.to_string(),
            contract_code_id: name_contract_code_id,
        });

        // instantiate aurans resolver contract
        let resolver_contract_addr = app
            .instantiate_contract(
                resolver_contract_code_id,
                Addr::unchecked(ADMIN),
                &ResolverInstantiateMsg {
                    admin: ADMIN.to_string(),
                    name_contract: name_contract_addr.to_string(),
                },
                &[],
                "test instantiate contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: resolver_contract_addr.to_string(),
            contract_code_id: resolver_contract_code_id,
        });

        (app, contract_info_vec)
    }

    #[test]
    fn test_instantiate_contracts() {
        let (_app, contract_info_vec) = instantiate_contracts();

        // check if all contracts are instantiated
        assert_eq!(contract_info_vec.len(), 3);
    }
}
