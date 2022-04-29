use frame_support::dispatch::GetStorageVersion;
use frame_support::traits::{PalletInfoAccess, PalletInfo};
use frame_support::weights::Weight;
use frame_system::Config as SystemConfig;
use crate::pallet::V1;

pub fn migrate_dao_keys<T, P>() -> Weight
where
    T: SystemConfig,
    P: GetStorageVersion + PalletInfoAccess,
{
    if P::current_storage_version() == V1 {

        Weight::max_value()
    } else {
        warn!("pallet-deip-dao: Tried to run migration but storage version is updated to V2. So this code probably needs to be removed");
        0
    }
}