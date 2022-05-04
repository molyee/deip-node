#![allow(type_alias_bounds)]
use crate::*;
use sp_runtime::traits::AtLeast32BitUnsigned;
use deip_serializable_u128::SerializableAtLeast32BitUnsigned;
use codec::{Encode, Decode};
use frame_support::{RuntimeDebug};
#[cfg(feature = "std")]
use serde::{self, Serialize, Deserialize};
use scale_info::TypeInfo;

use frame_support::traits::fungibles;
use sp_std::marker::PhantomData;
use sp_std::default::Default;

pub trait TransferUnitT<Account, Impl> {
    fn transfer(self, from: Account, to: Account);
}

pub trait GenericAssetT<Id, Payload, Account, Impl>: TransferUnitT<Account, Impl> + Sized {
    fn new(id: Id, payload: Payload) -> Self;
    fn id(&self) -> &Id;
    fn payload(&self) -> &Payload;
}

pub trait FTokenT<Id, Balance, Account, Impl>: GenericAssetT<Id, Balance, Account, Impl> {
    fn balance(id: Id, account: &Account) -> Self;
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct GenericAsset
    <Id, Payload, Account, Impl>
    (Id, Payload, PhantomData<(Account, Impl)>)
    where Self: GenericAssetT<Id, Payload, Account, Impl>;

impl<Id, Payload, Account, Impl>
    GenericAssetT<Id, Payload, Account, Impl>
    for GenericAsset<Id, Payload, Account, Impl>
{
    fn new(id: Id, payload: Payload) -> Self {
        Self(id, payload, <_>::default())
    }

    fn id(&self) -> &Id {
        &self.0
    }
    fn payload(&self) -> &Payload {
        &self.1
    }
}

impl<Id, Payload, Account, Impl>
    TransferUnitT<Account, Impl>
    for GenericAsset<Id, Payload, Account, Impl>
{
    fn transfer(self, _from: Account, _to: Account) {}
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct GenericFToken // type name
    <Id, Balance, Account, Impl> // type template
    (GenericAsset<Id, Balance, Account, Impl>) // type structure
    where Self: FTokenT<Id, Balance, Account, Impl>; // type class/signature

impl<Account, Impl: fungibles::Transfer<Account>>
    TransferUnitT<Account, Impl>
    for GenericFToken<Impl::AssetId, Impl::Balance, Account, Impl>
{
    fn transfer(self, from: Account, to: Account) {
        Impl::transfer(
            self.0.0,
            &from,
            &to,
            self.0.1,
            true
        ).unwrap();
    }
}

impl<Account, Impl: fungibles::Transfer<Account>>
    GenericAssetT<Impl::AssetId, Impl::Balance, Account, Impl>
    for GenericFToken<Impl::AssetId, Impl::Balance, Account, Impl>
{
    fn new(id: Impl::AssetId, payload: Impl::Balance) -> Self {
        Self(GenericAsset::new(id, payload))
    }
    fn id(&self) -> &Impl::AssetId {
        &self.0.id()
    }
    fn payload(&self) -> &Impl::Balance {
        &self.0.payload()
    }
}

impl<Account, Impl: fungibles::Transfer<Account>>
    FTokenT<Impl::AssetId, Impl::Balance, Account, Impl>
    for GenericFToken<Impl::AssetId, Impl::Balance, Account, Impl>
    where Impl::AssetId: Copy
{
    fn balance(id: Impl::AssetId, account: &Account) -> Self {
        Self::new(id, Impl::balance(id, account))
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
