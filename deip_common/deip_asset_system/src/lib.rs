#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

pub mod asset;
pub mod investment_opportunity;
pub mod asset_bus;

pub use deip_assets_error::{ReserveError, UnreserveError};
use frame_support::dispatch::Parameter;
use sp_runtime::traits::{AtLeast32BitUnsigned, Member};
use sp_std::prelude::*;

pub trait AssetIdInitT<AssetId> {
    fn asset_id(raw: &[u8]) -> AssetId;
}

pub trait DeipAssetSystem<AccountId, SourceId, InvestmentId>: AssetIdInitT<Self::AssetId> {
    /// The units in which asset balances are recorded.
    type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

    /// The arithmetic type of asset identifier.
    type AssetId: Member + Parameter + Default + Copy + AsRef<[u8]>;

    fn account_balance(account: &AccountId, asset: &Self::AssetId) -> Self::Balance;

    fn total_supply(asset: &Self::AssetId) -> Self::Balance;

    fn transactionally_transfer(
        from: &AccountId,
        asset: Self::AssetId,
        transfers: &[(Self::Balance, AccountId)],
    ) -> Result<(), ()>;

    /// Tries to transfer assets specified by `shares` from
    /// `account` to a specific balance identified by `id`.
    /// Some collateral fee may be locked from `account`.
    fn transactionally_reserve(
        account: &AccountId,
        id: InvestmentId,
        shares: &[(Self::AssetId, Self::Balance)],
        asset: Self::AssetId,
    ) -> Result<(), ReserveError<Self::AssetId>>;

    /// Transfers all assets currently owned by `id` to the account, used in
    /// transactionally_reserve, in a transactional way.
    fn transactionally_unreserve(id: InvestmentId) -> Result<(), UnreserveError<Self::AssetId>>;

    /// Transfers `amount` of assets `id` owned by account specified with `id` to `who`.
    fn transfer_from_reserved(
        from: InvestmentId,
        to: &AccountId,
        id: Self::AssetId,
        amount: Self::Balance,
    ) -> Result<(), UnreserveError<Self::AssetId>>;

    /// Transfers `amount` of assets `id` owned by account specified with `id` to `who`.
    fn transfer_from_reserved2<Unit: TransferUnitT>(
        from: InvestmentId,
        to: &AccountId,
        unit: Unit
    ) -> Result<(), UnreserveError<Self::AssetId>>
    {
        Transfer::new(from, to, unit);
        Ok(())
    }

    /// Transfers `amount` of assets from `who` to account specified by `id`.
    /// Assets should be specified in call to `transactionally_reserve`.
    fn transfer_to_reserved(
        who: &AccountId,
        id: InvestmentId,
        amount: Self::Balance,
    ) -> Result<(), UnreserveError<Self::AssetId>>;
}

#[allow(dead_code)]
pub struct TransferUnit<Id, Data> {
    id: Id,
    data: Data
}
pub trait TransferUnitT {
    type Id;
    fn id(&self) -> Self::Id;
}
#[allow(dead_code)]
pub struct Transfer<Unit: TransferUnitT, From, To> {
    from: From,
    to: To,
    unit: TransferUnit<Unit::Id, Unit>,
}
impl<Unit: TransferUnitT, From, To> TransferT<From, To> for Transfer<Unit, From, To> {
    type Unit = Unit;
}
pub trait TransferT<From, To> {
    type Unit: TransferUnitT;
    fn new(from: From, to: To, unit: Self::Unit) -> Transfer<Self::Unit, From, To> {
        Transfer {
            from,
            to,
            unit: TransferUnit { id: unit.id(), data: unit }
        }
    }
}
