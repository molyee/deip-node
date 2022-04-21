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

pub trait DeipAssetSystem<To, SourceId, From>: AssetIdInitT<Self::AssetId> {
    /// The units in which asset balances are recorded.
    type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

    /// The arithmetic type of asset identifier.
    type AssetId: Member + Parameter + Default + Copy + AsRef<[u8]>;

    fn account_balance(account: &To, asset: &Self::AssetId) -> Self::Balance;

    fn total_supply(asset: &Self::AssetId) -> Self::Balance;

    fn transactionally_transfer(
        from: &To,
        asset: Self::AssetId,
        transfers: &[(Self::Balance, To)],
    ) -> Result<(), ()>;

    /// Tries to transfer assets specified by `shares` from
    /// `account` to a specific balance identified by `id`.
    /// Some collateral fee may be locked from `account`.
    fn transactionally_reserve(
        account: &To,
        id: From,
        shares: &[(Self::AssetId, Self::Balance)],
        asset: Self::AssetId,
    ) -> Result<(), ReserveError<Self::AssetId>>;

    /// Transfers all assets currently owned by `id` to the account, used in
    /// transactionally_reserve, in a transactional way.
    fn transactionally_unreserve(id: From) -> Result<(), UnreserveError<Self::AssetId>>;

    /// Transfers `amount` of assets `id` owned by account specified with `id` to `who`.
    fn transfer_from_reserved(
        from: From,
        to: &To,
        id: Self::AssetId,
        amount: Self::Balance,
    ) -> Result<(), UnreserveError<Self::AssetId>>;

    /// Transfers `amount` of assets `id` owned by account specified with `id` to `who`.
    fn transfer<Unit: TransferUnitT<To>>(
        from: From,
        to: &To,
        unit: Unit
    ) -> Result<(), UnreserveError<Self::AssetId>>
        where From: TransferSourceT<To>,
              To: TransferTargetT<From> + Clone,
    {
        Transfer::new(from, to.clone(), unit).transfer();
        Ok(())
    }

    /// Transfers `amount` of assets from `who` to account specified by `id`.
    /// Assets should be specified in call to `transactionally_reserve`.
    fn transfer_to_reserved(
        who: &To,
        id: From,
        amount: Self::Balance,
    ) -> Result<(), UnreserveError<Self::AssetId>>;
}

pub trait TransferUnitT<Account> {
    // type Id;
    // fn id(&self) -> Self::Id;
    fn transfer(self, from: Account, to: Account);
}

#[allow(dead_code)]
pub struct Transfer<From, To, Unit: TransferUnitT<To>> {
    from: From,
    to: To,
    unit:  Unit,
}

impl
<
    From: TransferSourceT<To>,
    To: TransferTargetT<From>,
    Unit: TransferUnitT<To>
>
TransferT<From, To, Unit> for Transfer<From, To, Unit>
{
    fn new(from: From, to: To, unit: Unit) -> Self {
        Transfer {
            from,
            to,
            unit
        }
    }

    fn transfer(self) {
        let Self { from, to, unit } = self;
        unit.transfer(from.into_target(), to);
    }
}

pub trait TransferT<From, To, Unit>: Sized
{
    fn new(from: From, to: To, unit: Unit) -> Self;
    fn transfer(self);
}

pub trait TransferSourceT<Target: ?Sized + TransferTargetT<Self>> {
    fn into_target(self) -> Target;
}
pub trait TransferTargetT<Source: ?Sized + TransferSourceT<Self>> {
    fn into_source(self) -> Source;
}
