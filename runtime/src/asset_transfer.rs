#![allow(type_alias_bounds)]
use pallet_assets::Config;
use frame_system::RawOrigin;
use frame_support::dispatch::UnfilteredDispatchable;
use sp_runtime::traits::StaticLookup;
use deip_asset_system::{TransferUnitT, asset::GenericAsset};

/// Fungible token transfer
pub struct FTokenTransfer<T: Config>(GenericAsset<T::AssetId, T::Balance>);

impl<T: Config> TransferUnitT<T::AccountId> for FTokenTransfer<T> {

    fn transfer(self, from: T::AccountId, to: T::AccountId) {
        let call = pallet_assets::Call::<T>::transfer {
            id: self.0.0,
            target: T::Lookup::unlookup(to),
            amount: self.0.1
        };
        call.dispatch_bypass_filter(RawOrigin::Signed(from).into()).unwrap();
    }
}
