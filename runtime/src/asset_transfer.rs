use crate::{Runtime, DeipAssets};
use pallet_deip_investment_opportunity::{Config, module::{DeipAsset, DeipAssetId}};

pub struct DeipAssetTransfer<T: Config>(DeipAsset<T>);

impl<T: Config> deip_asset_system::TransferUnitT<T::AccountId> for DeipAssetTransfer<T> {
    type Id = DeipAssetId<T>;

    fn id(&self) -> Self::Id {
        *self.0.id()
    }

    fn transfer(self, from: T::AccountId, to: T::AccountId) {
        DeipAssets::transfer_from_reserved(from, to, self.id(), self.0.amount().clone());
    }
}
