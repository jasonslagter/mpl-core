use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::UpdatePluginAccounts,
    plugins::{Plugin, PluginType, ValidationResult},
    utils::fetch_core_data,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct UpdatePluginArgs {
    pub plugin: Plugin,
}

pub(crate) fn update_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: UpdatePluginArgs,
) -> ProgramResult {
    // Accounts.
    let ctx = UpdatePluginAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
    }

    let (asset, _, plugin_registry) = fetch_core_data(ctx.accounts.asset_address)?;
    let plugin_registry = plugin_registry.ok_or(MplCoreError::PluginsNotInitialized)?;

    let plugin_type: PluginType = (&args.plugin).into();
    let registry_record = plugin_registry
        .registry
        .iter()
        .find(|record| record.plugin_type == plugin_type)
        .ok_or(MplCoreError::PluginNotFound)?;

    let result = Plugin::load(ctx.accounts.asset_address, registry_record.offset)?
        .validate_update_plugin(&asset, &ctx.accounts, &args, &registry_record.authorities)?;
    if result == ValidationResult::Rejected {
        return Err(MplCoreError::InvalidAuthority.into());
    } else if result == ValidationResult::Approved {
        //TODO: Handle plugins that are dynamically sized.
        args.plugin
            .save(ctx.accounts.asset_address, registry_record.offset)?;
    } else {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    Ok(())
}
