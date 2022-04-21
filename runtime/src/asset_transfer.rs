// use crate::{Runtime, DeipAssets, AccountId};
use pallet_deip_investment_opportunity::{Config, module::{DeipAsset, DeipAssetId}};
use pallet_assets::{Config as AssetsConfig};
use frame_system::RawOrigin;
use frame_support::dispatch::UnfilteredDispatchable;
use sp_runtime::traits::StaticLookup;
use deip_asset_system::asset::{Asset, GenericAsset, GenericAssetT};

pub type AssetX<T> = GenericAsset<
    <T as AssetsConfig>::AssetId,
    <T as AssetsConfig>::Balance
>;

pub type _Asset<T> = Asset<<T as AssetsConfig>::AssetId, <T as AssetsConfig>::Balance>;

pub struct DeipAssetTransfer<T: Config + AssetsConfig>(AssetX<T>);

impl<T: Config + AssetsConfig> deip_asset_system::TransferUnitT<T::AccountId> for DeipAssetTransfer<T> {
    // type Id = <T as AssetsConfig>::AssetId;
    //
    // fn id(&self) -> Self::Id {
    //     *self.0.id()
    // }

    fn transfer(self, from: T::AccountId, to: T::AccountId) {
        // DeipAssets::transfer_from_reserved(from, &to, *self.0.id(), self.0.amount().clone());
        let call = pallet_assets::Call::<T>::transfer {
            id: self.0.0,
            target: T::Lookup::unlookup(to),
            amount: self.0.1
        };
        call.dispatch_bypass_filter(RawOrigin::Signed(from).into()).unwrap();
    }
}
