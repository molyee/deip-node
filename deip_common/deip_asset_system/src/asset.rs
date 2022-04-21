#![allow(type_alias_bounds)]
use crate::*;
use sp_runtime::traits::AtLeast32BitUnsigned;
use deip_serializable_u128::SerializableAtLeast32BitUnsigned;
use codec::{Encode, Decode};
use frame_support::{RuntimeDebug};
#[cfg(feature = "std")]
use serde::{self, Serialize, Deserialize};
use scale_info::TypeInfo;

use crate::{TransferUnitT};
use frame_support::traits::fungibles::Transfer;

pub trait GenericAssetT<Id, Payload> {}
pub struct GenericAsset<Id, Payload>(pub Id, pub Payload) where Self: GenericAssetT<Id, Payload>;
impl<Id, Payload> GenericAssetT<Id, Payload> for GenericAsset<Id, Payload> {}

struct GenericFToken<Id, Balance>(GenericAsset<Id, Balance>);

impl<T: Transfer<Account>, Account> TransferUnitT<Account, T> for GenericFToken<T::AssetId, T::Balance> {
    fn transfer(self, from: Account, to: Account) {
        T::transfer(
            self.0.0,
            &from,
            &to,
            self.0.1,
            true
        ).unwrap();
    }
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct Asset<AssetId, AssetBalance: Clone + AtLeast32BitUnsigned> {
    id: AssetId,
    amount: SerializableAtLeast32BitUnsigned<AssetBalance>,
}

impl<AssetId, AssetBalance: Clone + AtLeast32BitUnsigned> Asset<AssetId, AssetBalance> {
    pub fn new(id: AssetId, amount: AssetBalance) -> Self {
        Self { id, amount: SerializableAtLeast32BitUnsigned(amount) }
    }

    pub fn id(&self) -> &AssetId {
        &self.id
    }

    pub fn amount(&self) -> &AssetBalance {
        &self.amount.0
    }
}
