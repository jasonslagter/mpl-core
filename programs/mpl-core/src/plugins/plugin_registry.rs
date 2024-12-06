use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};
use std::{cmp::Ordering, collections::BTreeMap};

use crate::{
    error::MplCoreError,
    plugins::validate_lifecycle_checks,
    state::{Authority, DataBlob, Key, SolanaAccount},
};

use super::{
    CheckResult, ExternalCheckResult, ExternalCheckResultBits, ExternalPluginAdapterKey,
    ExternalPluginAdapterType, ExternalPluginAdapterUpdateInfo, HookableLifecycleEvent, PluginType,
};

/// The Plugin Registry stores a record of all plugins, their location, and their authorities.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct PluginRegistryV1 {
    /// The Discriminator of the header which doubles as a plugin metadata version.
    pub key: Key, // 1
    /// The registry of all plugins.
    pub registry: Vec<RegistryRecord>, // 4
    /// The registry of all adapter, third party, plugins.
    pub external_registry: Vec<ExternalRegistryRecord>, // 4
}

impl PluginRegistryV1 {
    /// Evaluate checks for all plugins in the registry.
    pub(crate) fn check_registry(
        &self,
        key: Key,
        check_fp: fn(&PluginType) -> CheckResult,
        result: &mut BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)>,
    ) {
        for record in &self.registry {
            result.insert(
                record.plugin_type,
                (key, check_fp(&record.plugin_type), record.clone()),
            );
        }
    }

    pub(crate) fn check_adapter_registry(
        &self,
        account: &AccountInfo,
        key: Key,
        lifecycle_event: &HookableLifecycleEvent,
        result: &mut BTreeMap<
            ExternalPluginAdapterKey,
            (Key, ExternalCheckResultBits, ExternalRegistryRecord),
        >,
    ) -> ProgramResult {
        for record in &self.external_registry {
            if let Some(lifecycle_checks) = &record.lifecycle_checks {
                for (event, check_result) in lifecycle_checks {
                    if event == lifecycle_event {
                        let plugin_key = ExternalPluginAdapterKey::from_record(account, record)?;

                        result.insert(
                            plugin_key,
                            (
                                key,
                                ExternalCheckResultBits::from(*check_result),
                                record.clone(),
                            ),
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Increase the offsets of all plugins after a certain offset.
    pub(crate) fn bump_offsets(&mut self, offset: usize, size_diff: isize) -> ProgramResult {
        for record in &mut self.registry {
            if record.offset > offset {
                record.offset = (record.offset as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?
                    as usize;
            }
        }

        for record in &mut self.external_registry {
            if record.offset > offset {
                solana_program::msg!("Bumping Record: {:?}", record);
                record.offset = (record.offset as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?
                    as usize;

                if let Some(data_offset) = record.data_offset {
                    if data_offset > offset {
                        solana_program::msg!("Bumping Data: {:?}", record);
                        record.data_offset = Some(
                            (data_offset as isize)
                                .checked_add(size_diff)
                                .ok_or(MplCoreError::NumericalOverflow)?
                                as usize,
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

impl DataBlob for PluginRegistryV1 {
    const BASE_LEN: usize = 1 // Key
     + 4 // Registry Length
     + 4; // External Registry Length

    fn len(&self) -> usize {
        Self::BASE_LEN
            + self
                .registry
                .iter()
                .map(|record| record.len())
                .sum::<usize>()
            + self
                .external_registry
                .iter()
                .map(|record| record.len())
                .sum::<usize>()
    }
}

impl SolanaAccount for PluginRegistryV1 {
    fn key() -> Key {
        Key::PluginRegistryV1
    }
}

/// A simple type to store the mapping of plugin type to plugin data.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct RegistryRecord {
    /// The type of plugin.
    pub plugin_type: PluginType, // 1
    /// The authority who has permission to utilize a plugin.
    pub authority: Authority, // Variable
    /// The offset to the plugin in the account.
    pub offset: usize, // 8
}

impl RegistryRecord {
    /// Associated function for sorting `RegistryRecords` by offset.
    pub fn compare_offsets(a: &RegistryRecord, b: &RegistryRecord) -> Ordering {
        a.offset.cmp(&b.offset)
    }
}

impl DataBlob for RegistryRecord {
    const BASE_LEN: usize = PluginType::BASE_LEN
     + Authority::BASE_LEN // Authority Discriminator
      + 8; // Offset

    fn len(&self) -> usize {
        self.plugin_type.len() + self.authority.len() + 8
    }
}

/// A type to store the mapping of third party plugin type to third party plugin header and data.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub struct ExternalRegistryRecord {
    /// The adapter, third party plugin type.
    pub plugin_type: ExternalPluginAdapterType,
    /// The authority of the external plugin adapter.
    pub authority: Authority,
    /// The lifecyle events for which the the external plugin adapter is active.
    pub lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalCheckResult)>>,
    /// The offset to the plugin in the account.
    pub offset: usize, // 8
    /// For plugins with data, the offset to the data in the account.
    pub data_offset: Option<usize>,
    /// For plugins with data, the length of the data in the account.
    pub data_len: Option<usize>,
}

impl ExternalRegistryRecord {
    /// Update the adapter registry record with the new info, if relevant.
    pub fn update(&mut self, update_info: &ExternalPluginAdapterUpdateInfo) -> ProgramResult {
        match update_info {
            ExternalPluginAdapterUpdateInfo::LifecycleHook(update_info) => {
                if let Some(checks) = &update_info.lifecycle_checks {
                    validate_lifecycle_checks(checks, false)?;
                    self.lifecycle_checks
                        .clone_from(&update_info.lifecycle_checks)
                }
            }
            ExternalPluginAdapterUpdateInfo::Oracle(update_info) => {
                if let Some(checks) = &update_info.lifecycle_checks {
                    validate_lifecycle_checks(checks, true)?;
                    self.lifecycle_checks
                        .clone_from(&update_info.lifecycle_checks)
                }
            }
            _ => (),
        }

        Ok(())
    }
}

impl DataBlob for ExternalRegistryRecord {
    const BASE_LEN: usize = ExternalPluginAdapterType::BASE_LEN
     + Authority::BASE_LEN // Authority Discriminator
      + 1 // Lifecycle checks option
      + 8 // Offset
      + 1 // Data offset option
      + 1; // Data len option

    fn len(&self) -> usize {
        let mut len = self.plugin_type.len() + self.authority.len() + 1 + 8 + 1 + 1;

        if let Some(checks) = &self.lifecycle_checks {
            len += 4 // 4 bytes for the length of the checks vector
                + checks.len()
                * (HookableLifecycleEvent::BASE_LEN + ExternalCheckResult::BASE_LEN);
        }

        if self.data_offset.is_some() {
            len += 8;
        }

        if self.data_len.is_some() {
            len += 8;
        }

        len
    }
}
