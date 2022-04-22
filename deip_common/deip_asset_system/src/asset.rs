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
use frame_support::traits::fungibles;
use sp_std::marker::PhantomData;
use sp_std::default::Default;

pub trait GenericAssetT<Id, Payload, Account, Transfer>: TransferUnitT<Account, Transfer> + Sized {
    fn new(id: Id, payload: Payload) -> Self;
}

pub struct GenericAsset
    <Id, Payload, Account, Transfer>
    (Id, Payload, PhantomData<(Account, Transfer)>)
    where Self: GenericAssetT<Id, Payload, Account, Transfer>;

impl<Id, Payload, Account, Transfer>
    GenericAssetT<Id, Payload, Account, Transfer>
    for GenericAsset<Id, Payload, Account, Transfer>
{
    fn new(id: Id, payload: Payload) -> Self {
        Self(id, payload, <_>::default())
    }
}

impl<Id, Payload, Account, Transfer>
    TransferUnitT<Account, Transfer>
    for GenericAsset<Id, Payload, Account, Transfer>
{
    fn transfer(self, _from: Account, _to: Account) {}
}

pub struct GenericFToken // type name
    <Account, T: fungibles::Inspect<Account>> // type template
    (GenericAsset<T::AssetId, T::Balance, Account, T>) // type structure
    where Self: GenericAssetT<T::AssetId, T::Balance, Account, T>; // type class/signature

impl<Account, T: fungibles::Transfer<Account>>
    GenericAssetT<T::AssetId, T::Balance, Account, T>
    for GenericFToken<Account, T>
{
    fn new(id: T::AssetId, payload: T::Balance) -> Self {
        Self(GenericAsset::new(id, payload))
    }
}

impl<Account, Transfer: fungibles::Transfer<Account>>
    TransferUnitT<Account, Transfer>
    for GenericFToken<Account, Transfer>
{
    fn transfer(self, from: Account, to: Account) {
        Transfer::transfer(
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
