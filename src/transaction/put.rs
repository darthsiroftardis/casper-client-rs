use async_trait::async_trait;
use clap::{ArgMatches, Command};
use serde::{Deserialize, Serialize};

use casper_client::cli::{parse, CliError, PublicKey, TransactionBuilderParams, TransactionStrParams, get_maybe_secret_key, query_global_state};
use casper_types::{ActivationPoint, CLValue, CoreConfig, HighwayConfig, ProtocolVersion, PublicKey, StorageCosts, SystemConfig, TransactionConfig, VacancyConfig, WasmConfig, U512, HashAddr, Key, SystemHashRegistry};
use casper_client::Verbosity;

use super::creation_common::{
    activate_bid, add_bid, add_reservations, cancel_reservations, change_bid_public_key, delegate,
    invocable_entity, invocable_entity_alias, min_bid_override, package, package_alias, public_key,
    redelegate, session, transfer, undelegate, withdraw_bid, withdraw_bid_all,
};

use crate::{command::ClientCommand, common, Success};

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
// Disallow unknown fields to ensure config files and command-line overrides contain valid keys.
#[serde(deny_unknown_fields)]
struct TomlNetwork {
    name: String,
    maximum_net_message_size: u32,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
// Disallow unknown fields to ensure config files and command-line overrides contain valid keys.
#[serde(deny_unknown_fields)]
struct TomlProtocol {
    version: ProtocolVersion,
    hard_reset: bool,
    activation_point: ActivationPoint,
}

/// A chainspec configuration as laid out in the TOML-encoded configuration file.
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
// Disallow unknown fields to ensure config files and command-line overrides contain valid keys.
#[serde(deny_unknown_fields)]
pub(super) struct TomlChainspec {
    protocol: TomlProtocol,
    network: TomlNetwork,
    core: CoreConfig,
    transactions: TransactionConfig,
    highway: HighwayConfig,
    wasm: WasmConfig,
    system_costs: SystemConfig,
    vacancy: VacancyConfig,
    storage_costs: StorageCosts,
}

pub struct PutTransaction;
const ALIAS: &str = "put-txn";
#[async_trait]
impl ClientCommand for PutTransaction {
    const NAME: &'static str = "put-transaction";

    const ABOUT: &'static str = "Create a transaction and send it to the network for execution";

    fn build(display_order: usize) -> Command {
        Command::new(Self::NAME)
            .about(Self::ABOUT)
            .alias(ALIAS)
            .subcommand_required(true)
            .subcommand(add_bid::put_transaction_build())
            .subcommand(activate_bid::put_transaction_build())
            .subcommand(withdraw_bid_all::put_transaction_build())
            .subcommand(withdraw_bid::put_transaction_build())
            .subcommand(delegate::put_transaction_build())
            .subcommand(undelegate::put_transaction_build())
            .subcommand(redelegate::put_transaction_build())
            .subcommand(change_bid_public_key::put_transaction_build())
            .subcommand(add_reservations::put_transaction_build())
            .subcommand(cancel_reservations::put_transaction_build())
            .subcommand(invocable_entity::put_transaction_build())
            .subcommand(invocable_entity_alias::put_transaction_build())
            .subcommand(package::put_transaction_build())
            .subcommand(package_alias::put_transaction_build())
            .subcommand(session::put_transaction_build())
            .subcommand(transfer::put_transaction_build())
            .display_order(display_order)
    }

    async fn run(matches: &ArgMatches) -> Result<Success, CliError> {
        match matches.subcommand() {
            None => Err(CliError::InvalidArgument {
                context: "Make Transaction",
                error: "failure to provide recognized subcommand".to_string(),
            }),
            Some((subcommand, arg_matches)) => match subcommand {
                add_bid::NAME => put_add_bid_transaction(arg_matches).await,
                activate_bid::NAME => put_activate_bid_transaction(arg_matches).await,
                withdraw_bid_all::NAME => put_withdraw_all_transaction(arg_matches).await,
                withdraw_bid::NAME => put_withdraw_bid_transaction(arg_matches).await,
                delegate::NAME => put_delegate_transaction(arg_matches).await,
                undelegate::NAME => put_undelegate_transaction(arg_matches).await,
                redelegate::NAME => put_redelegate_transaction(arg_matches).await,
                change_bid_public_key::NAME => put_change_public_key_transaction(arg_matches).await,
                add_reservations::NAME => put_add_reservations_transaction(arg_matches).await,
                cancel_reservations::NAME => put_cancel_reservations_transaction(arg_matches).await,
                invocable_entity::NAME => put_entity_by_hash_transaction(arg_matches).await,
                invocable_entity_alias::NAME => put_entity_by_name_transaction(arg_matches).await,
                package::NAME => put_by_package_hash_transaction(arg_matches).await,
                package_alias::NAME => put_package_by_name_transaction(arg_matches).await,
                session::NAME => put_session_transaction(arg_matches).await,
                transfer::NAME => put_transfer_transaction(arg_matches).await,
                _ => {
                    return Err(CliError::InvalidArgument {
                        context: "Make Transaction",
                        error: "failure to provide recognized subcommand".to_string(),
                    })
                }
            },
        }
    }
}

async fn put_add_bid_transaction(matches: &ArgMatches) -> Result<Success, CliError> {
    let node_address = common::node_address::get(matches);
    let rpc_id = common::rpc_id::get(matches);
    let verbosity_level = common::verbose::get(matches);

    let (transaction_builder_params, transaction_str_params) = add_bid::run(matches)?;
    casper_client::cli::put_transaction(
        rpc_id,
        node_address,
        verbosity_level,
        transaction_builder_params,
        transaction_str_params,
    )
    .await
    .map(Success::from)
}

async fn put_activate_bid_transaction(matches: &ArgMatches) -> Result<Success, CliError> {
    let node_address = common::node_address::get(matches);
    let rpc_id = common::rpc_id::get(matches);
    let verbosity_level = common::verbose::get(matches);

    let (transaction_builder_params, transaction_str_params) = activate_bid::run(matches)?;
    casper_client::cli::put_transaction(
        rpc_id,
        node_address,
        verbosity_level,
        transaction_builder_params,
        transaction_str_params,
    )
    .await
    .map(Success::from)
}

async fn put_withdraw_all_transaction(matches: &ArgMatches) -> Result<Success, CliError> {
    let node_address = common::node_address::get(matches);
    let rpc_id = common::rpc_id::get(matches);
    let verbosity_level = common::verbose::get(matches);

    let public_key_str = public_key::get(matches)?;
    let public_key = public_key::parse_public_key(&public_key_str)?;

    let (transaction_builder_params, transaction_str_params) =
        match casper_client::cli::get_auction_info("", node_address, verbosity_level, "")
            .await?
            .result
            .auction_state
            .bids()
            .find(|(bid_key, _bid)| **bid_key == public_key)
        {
            Some((_, bid)) => {
                let staked_amount = *bid.staked_amount();
                withdraw_bid_all::run(matches, staked_amount)?
            }
            None => return Err(CliError::FailedToGetAuctionState),
        };

    casper_client::cli::put_transaction(
        rpc_id,
        node_address,
        verbosity_level,
        transaction_builder_params,
        transaction_str_params,
    )
    .await
    .map(Success::from)
}

async fn put_withdraw_bid_transaction(matches: &ArgMatches) -> Result<Success, CliError> {
    let node_address = common::node_address::get(matches);
    let rpc_id = common::rpc_id::get(matches);
    let verbosity_level = common::verbose::get(matches);

    let (transaction_builder_params, transaction_str_params) = withdraw_bid::run(matches)?;

    if let TransactionBuilderParams::WithdrawBid {
        public_key,
        amount,
        min_bid_override,
    } = &transaction_builder_params
    {
        do_withdraw_amount_checks(node_address, verbosity_level, public_key.clone(), amount.clone(), *min_bid_override)?;
    }

    casper_client::cli::put_transaction(
        rpc_id,
        node_address,
        verbosity_level,
        transaction_builder_params,
        transaction_str_params,
    )
    .await
    .map(Success::from)
}

async fn put_delegate_transaction(arg_matches: &ArgMatches) -> Result<Success, CliError> {
    let node_address = common::node_address::get(arg_matches);
    let rpc_id = common::rpc_id::get(arg_matches);
    let verbosity_level = common::verbose::get(arg_matches);

    let (transaction_builder_params, transaction_str_params) = delegate::run(arg_matches)?;
    casper_client::cli::put_transaction(
        rpc_id,
        node_address,
        verbosity_level,
        transaction_builder_params,
        transaction_str_params,
    )
    .await
    .map(Success::from)
}

async fn put_undelegate_transaction(arg_matches: &ArgMatches) -> Result<Success, CliError> {
    let node_address = common::node_address::get(arg_matches);
    let rpc_id = common::rpc_id::get(arg_matches);
    let verbosity_level = common::verbose::get(arg_matches);

    let (transaction_builder_params, transaction_str_params) = undelegate::run(arg_matches)?;
    casper_client::cli::put_transaction(
        rpc_id,
        node_address,
        verbosity_level,
        transaction_builder_params,
        transaction_str_params,
    )
    .await
    .map(Success::from)
}

async fn put_redelegate_transaction(arg_matches: &ArgMatches) -> Result<Success, CliError> {
    let node_address = common::node_address::get(arg_matches);
    let rpc_id = common::rpc_id::get(arg_matches);
    let verbosity_level = common::verbose::get(arg_matches);

    let (transaction_builder_params, transaction_str_params) = redelegate::run(arg_matches)?;
    casper_client::cli::put_transaction(
        rpc_id,
        node_address,
        verbosity_level,
        transaction_builder_params,
        transaction_str_params,
    )
    .await
    .map(Success::from)
}

async fn put_change_public_key_transaction(arg_matches: &ArgMatches) -> Result<Success, CliError> {
    let node_address = common::node_address::get(arg_matches);
    let rpc_id = common::rpc_id::get(arg_matches);
    let verbosity_level = common::verbose::get(arg_matches);

    let (transaction_builder_params, transaction_str_params) =
        change_bid_public_key::run(arg_matches)?;
    casper_client::cli::put_transaction(
        rpc_id,
        node_address,
        verbosity_level,
        transaction_builder_params,
        transaction_str_params,
    )
    .await
    .map(Success::from)
}

async fn put_add_reservations_transaction(arg_matches: &ArgMatches) -> Result<Success, CliError> {
    let node_address = common::node_address::get(arg_matches);
    let rpc_id = common::rpc_id::get(arg_matches);
    let verbosity_level = common::verbose::get(arg_matches);

    let (transaction_builder_params, transaction_str_params) = add_reservations::run(arg_matches)?;
    casper_client::cli::put_transaction(
        rpc_id,
        node_address,
        verbosity_level,
        transaction_builder_params,
        transaction_str_params,
    )
    .await
    .map(Success::from)
}
async fn put_cancel_reservations_transaction(
    arg_matches: &ArgMatches,
) -> Result<Success, CliError> {
    let node_address = common::node_address::get(arg_matches);
    let rpc_id = common::rpc_id::get(arg_matches);
    let verbosity_level = common::verbose::get(arg_matches);

    let (transaction_builder_params, transaction_str_params) =
        cancel_reservations::run(arg_matches)?;
    casper_client::cli::put_transaction(
        rpc_id,
        node_address,
        verbosity_level,
        transaction_builder_params,
        transaction_str_params,
    )
    .await
    .map(Success::from)
}

async fn put_entity_by_hash_transaction(arg_matches: &ArgMatches) -> Result<Success, CliError> {
    let node_address = common::node_address::get(arg_matches);
    let rpc_id = common::rpc_id::get(arg_matches);
    let verbosity_level = common::verbose::get(arg_matches);

    let (transaction_builder_params, transaction_str_params) = invocable_entity::run(arg_matches)?;
    if let TransactionBuilderParams::InvocableEntity { entity_hash, entry_point , .. } = transaction_builder_params {
        let hash_addr = entity_hash.value();
        let entry_point = entry_point.to_string();
        let min_bid_override = min_bid_override::get(arg_matches);
        let args_as_json = transaction_str_params.session_args_json.to_string();
        let simple_args = transaction_str_params.session_args_simple.clone();
        check_auction_state_for_withdraw(node_address, verbosity_level,
            hash_addr, min_bid_override,  entry_point, args_as_json, simple_args,
        ).await?
    }

    casper_client::cli::put_transaction(
        rpc_id,
        node_address,
        verbosity_level,
        transaction_builder_params,
        transaction_str_params,
    )
    .await
    .map(Success::from)
}

async fn put_entity_by_name_transaction(arg_matches: &ArgMatches) -> Result<Success, CliError> {
    let node_address = common::node_address::get(arg_matches);
    let rpc_id = common::rpc_id::get(arg_matches);
    let verbosity_level = common::verbose::get(arg_matches);

    let (transaction_builder_params, transaction_str_params) =
        invocable_entity_alias::run(arg_matches)?;
    if let TransactionBuilderParams::InvocableEntityAlias { entity_alias, entry_point, .. } = transaction_builder_params {
        let account_key = {
            let secret_key = get_maybe_secret_key(transaction_str_params.secret_key, false, "")
                .unwrap()
                .unwrap();
            Key::Account(PublicKey::from(&secret_key)
                .to_account_hash())
        };
        let cl_value = casper_client::cli::query_global_state(rpc_id, node_address, verbosity_level, "", "", &account_key.to_formatted_string(), entity_alias)
            .await?
            .result
            .stored_value
            .as_cl_value()
            .unwrap();
        let key = CLValue::to_t::<Key>(cl_value).map_err(|err| CliError::InvalidCLValue(err.to_string()))?;
        match key {
            Key::Hash(hash_addr) => {
                let entry_point = entry_point.to_string();
                let min_bid_override = min_bid_override::get(arg_matches);
                let args_as_json = transaction_str_params.session_args_json.to_string();
                let simple_args = transaction_str_params.session_args_simple.clone();
                check_auction_state_for_withdraw(node_address, verbosity_level,
                                                 hash_addr, min_bid_override,  entry_point, args_as_json, simple_args,
                ).await?
            }
            Key::AddressableEntity(addr) => {
                let hash_addr = addr.value();
                let entry_point = entry_point.to_string();
                let min_bid_override = min_bid_override::get(arg_matches);
                let args_as_json = transaction_str_params.session_args_json.to_string();
                let simple_args = transaction_str_params.session_args_simple.clone();
                check_auction_state_for_withdraw(node_address, verbosity_level,
                                                 hash_addr, min_bid_override,  entry_point, args_as_json, simple_args,
                ).await?
            }
            _ => {}
        }
    }

    casper_client::cli::put_transaction(
        rpc_id,
        node_address,
        verbosity_level,
        transaction_builder_params,
        transaction_str_params,
    )
    .await
    .map(Success::from)
}

async fn put_by_package_hash_transaction(arg_matches: &ArgMatches) -> Result<Success, CliError> {
    let node_address = common::node_address::get(arg_matches);
    let rpc_id = common::rpc_id::get(arg_matches);
    let verbosity_level = common::verbose::get(arg_matches);

    let (transaction_builder_params, transaction_str_params) = package::run(arg_matches)?;
    if let TransactionBuilderParams::PackageWithMajorVersion { package_hash, entry_point, .. } = transaction_builder_params {
        let hash_addr = package_hash.value();
        let entry_point = entry_point.to_string();
        let min_bid_override = min_bid_override::get(arg_matches);
        let args_as_json = transaction_str_params.session_args_json.to_string();
        let simple_args = transaction_str_params.session_args_simple.clone();
        check_auction_state_for_withdraw(node_address, verbosity_level, hash_addr, min_bid_override,  entry_point, args_as_json, simple_args,
        ).await?
    }

    casper_client::cli::put_transaction(
        rpc_id,
        node_address,
        verbosity_level,
        transaction_builder_params,
        transaction_str_params,
    )
    .await
    .map(Success::from)
}

async fn put_package_by_name_transaction(arg_matches: &ArgMatches) -> Result<Success, CliError> {
    let node_address = common::node_address::get(arg_matches);
    let rpc_id = common::rpc_id::get(arg_matches);
    let verbosity_level = common::verbose::get(arg_matches);

    let (transaction_builder_params, transaction_str_params) = package_alias::run(arg_matches)?;
    if let TransactionBuilderParams::PackageAliasWithMajorVersion { package_alias, entry_point , .. } = transaction_builder_params {
        let account_key = {
            let secret_key = get_maybe_secret_key(transaction_str_params.secret_key, false, "")
                .unwrap()
                .unwrap();
            Key::Account(PublicKey::from(&secret_key)
                .to_account_hash())
        };
        let cl_value = casper_client::cli::query_global_state(rpc_id, node_address, verbosity_level, "", "", &account_key.to_formatted_string(), package_alias)
            .await?
            .result
            .stored_value
            .as_cl_value()
            .unwrap();
        let key = CLValue::to_t::<Key>(cl_value).map_err(|err| CliError::InvalidCLValue(err.to_string()))?;
        match key {
            Key::Hash(hash_addr) => {
                let entry_point = entry_point.to_string();
                let min_bid_override = min_bid_override::get(arg_matches);
                let args_as_json = transaction_str_params.session_args_json.to_string();
                let simple_args = transaction_str_params.session_args_simple.clone();
                check_auction_state_for_withdraw(node_address, verbosity_level, hash_addr, min_bid_override,  entry_point, args_as_json, simple_args,
                ).await?
            }
            Key::SmartContract(addr) => {
                let hash_addr = addr;
                let entry_point = entry_point.to_string();
                let min_bid_override = min_bid_override::get(arg_matches);
                let args_as_json = transaction_str_params.session_args_json.to_string();
                let simple_args = transaction_str_params.session_args_simple.clone();
                check_auction_state_for_withdraw(node_address, verbosity_level, hash_addr, min_bid_override,  entry_point, args_as_json, simple_args,
                ).await?
            }
            _ => {}
        }
    }


    casper_client::cli::put_transaction(
        rpc_id,
        node_address,
        verbosity_level,
        transaction_builder_params,
        transaction_str_params,
    )
    .await
    .map(Success::from)
}

async fn put_session_transaction(arg_matches: &ArgMatches) -> Result<Success, CliError> {
    let node_address = common::node_address::get(arg_matches);
    let rpc_id = common::rpc_id::get(arg_matches);
    let verbosity_level = common::verbose::get(arg_matches);

    let (transaction_builder_params, transaction_str_params) = session::run(arg_matches)?;
    casper_client::cli::put_transaction(
        rpc_id,
        node_address,
        verbosity_level,
        transaction_builder_params,
        transaction_str_params,
    )
    .await
    .map(Success::from)
}

async fn put_transfer_transaction(arg_matches: &ArgMatches) -> Result<Success, CliError> {
    let node_address = common::node_address::get(arg_matches);
    let rpc_id = common::rpc_id::get(arg_matches);
    let verbosity_level = common::verbose::get(arg_matches);

    let (transaction_builder_params, transaction_str_params) = transfer::run(arg_matches)?;
    casper_client::cli::put_transaction(
        rpc_id,
        node_address,
        verbosity_level,
        transaction_builder_params,
        transaction_str_params,
    )
    .await
    .map(Success::from)
}

async fn do_withdraw_amount_checks(
    node_address: &str,
    verbosity_level: u64,
    public_key: PublicKey,
    amount: U512,
    min_bid_override: bool,
) -> Result<(), CliError> {
    let chainspec_bytes = casper_client::cli::get_chainspec("", node_address, verbosity_level)
        .await?
        .result
        .chainspec_bytes;

    let chainspec_as_str = std::str::from_utf8(chainspec_bytes.chainspec_bytes()).unwrap();
    let toml_chainspec: TomlChainspec = toml::from_str(chainspec_as_str).unwrap();

    let minimum_validator_bid = toml_chainspec.core.minimum_bid_amount;

    match casper_client::cli::get_auction_info("", node_address, verbosity_level, "")
        .await?
        .result
        .auction_state
        .bids()
        .find(|(bid_key, _bid)| **bid_key == *public_key)
    {
        Some((_, bid)) => {
            let staked_amount = *bid.staked_amount();
            let remainder = staked_amount.saturating_sub(*amount);
            if remainder < U512::from(minimum_validator_bid) {
                if !min_bid_override {
                    return Err(CliError::ReducedStakeBelowMinAmount);
                } else {
                    println!("[WARN] Execution of this withdraw bid will result in unbonding of all stake")
                }
            }
        }
        None => return Err(CliError::FailedToGetAuctionState),
    };

    Ok(())
}

async fn check_auction_state_for_withdraw(
    node_address: &str,
    verbosity_level: u64,
    hash_addr: HashAddr,
    min_bid_override: bool,
    entry_point_name: String,
    session_args_as_json: String,
    session_args_simple: Vec<&str>,
) -> Result<(), CliError> {
    // Best guess on the entry point name
    if entry_point_name == "withdraw".to_string() {
        let registry =
            casper_client::cli::get_system_contract_registry("", node_address, verbosity_level)
                .await?;
        let auction_hash_addr = *registry
            .get("auction")
            .ok_or_else(CliError::MissingAuctionHash)?;
        // First check if we are calling the auction.
        if auction_hash_addr == hash_addr {
            // Now parse the args for the amount to do the value check.
        } else {
            // check if the hash addr matches the package hash addr on the contract itself.
            let key = Key::Hash(auction_hash_addr);
            let package_addr = query_global_state("", node_address, verbosity_level, "", "", &key, "")
                .await?
                .result
                .stored_value
                .as_contract()
                .ok_or_else(CliError::FailedToGetSystemHashRegistry)?.contract_package_hash().value();
            if (package_addr != hash_addr) {
                return Ok(())
            }
        }

        if let Some(runtime_args) =
            parse::args_json::session::parse(&session_args_as_json)?
        {
            match runtime_args.get("amount").map(|cl| {
                CLValue::to_t::<U512>(cl)
                    .map_err(|err| CliError::InvalidCLValue(err.to_string()))?
            }) {
                Some(amount) => {
                    let public_key = runtime_args
                        .get("public_key")
                        .map(|cl| {
                            CLValue::to_t::<PublicKey>(cl)
                                .map_err(|err| CliError::InvalidCLValue(err.to_string()))?
                        })
                        .unwrap();
                    return do_withdraw_amount_checks(
                        node_address,
                        verbosity_level,
                        public_key,
                        amount,
                        min_bid_override,
                    )
                        .await;
                }
                None => {
                    println!("no amount arg found, skipping withdraw check")
                }
            };
        };
        if let Some(runtime_args) =
            parse::arg_simple::session::parse(&session_args_simple)?
        {
            match runtime_args.get("amount").map(|cl| {
                CLValue::to_t::<U512>(cl)
                    .map_err(|err| CliError::InvalidCLValue(err.to_string()))?
            }) {
                Some(amount) => {
                    let public_key = runtime_args
                        .get("public_key")
                        .map(|cl| {
                            CLValue::to_t::<PublicKey>(cl)
                                .map_err(|err| CliError::InvalidCLValue(err.to_string()))?
                        })
                        .unwrap();
                    return do_withdraw_amount_checks(
                        node_address,
                        verbosity_level,
                        public_key,
                        amount,
                        min_bid_override,
                    )
                        .await
                }
                None => {
                    println!("no amount arg found, skipping withdraw check")
                }
            };
        };
    }
    Ok(())
}
