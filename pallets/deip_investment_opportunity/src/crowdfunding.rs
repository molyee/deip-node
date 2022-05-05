use codec::{Encode, Decode};
#[cfg(feature = "std")]
use serde::{self, Serialize, Deserialize};
use sp_runtime::traits::{AtLeast32BitUnsigned, Saturating};
use frame_support::{RuntimeDebug, ensure};
use scale_info::TypeInfo;
use sp_std::prelude::*;
use sp_std::default::Default;
use deip_serializable_u128::SerializableAtLeast32BitUnsigned;
use deip_asset_system::asset::{Asset, FTokenT, GenericAssetT};
use deip_transaction_ctx::{TransactionCtxId, TransactionCtxT};

use crate::module::{FToken, FTokenId, FTokenBalance};
use crate::{SimpleCrowdfundingMapV2};

impl<T: crate::Config> CrowdfundingT<T>
    for SimpleCrowdfundingV2<
        T::AccountId,
        T::Moment,
        FTokenId<T>,
        FTokenBalance<T>,
        TransactionCtxId<T::TransactionCtx>
    >
    where
        SimpleCrowdfunding<
            T::Moment,
            FTokenId<T>,
            FTokenBalance<T>,
            TransactionCtxId<T::TransactionCtx>
        >: CrowdfundingT<T>
{
    fn new(
        ctx: T::TransactionCtx,
        creator: T::AccountId,
        account: T::AccountId,
        external_id: InvestmentId,
        start_time: T::Moment,
        end_time: T::Moment,
        asset_id: FTokenId<T>,
        soft_cap: FTokenBalance<T>,
        hard_cap: FTokenBalance<T>,
        shares: Vec<FToken<T>>
    ) -> Self
    {
        SimpleCrowdfundingV2 {
            v1: SimpleCrowdfunding::new(
                ctx,
                Default::default(),
                Default::default(),
                external_id,
                start_time,
                end_time,
                asset_id,
                soft_cap,
                hard_cap,
                shares
            ),
            creator,
            account,
            registered_shares: 0,
        }
    }

    fn id(&self) -> &InvestmentId {
        self.v1.id()
    }

    fn register_share(&mut self) -> Result<(), SimpleCrowdfundingStatus> {
        if all_shares_registered::<T>(self) {
            return Err(self.v1.status)
        }

        self.registered_shares += 1;

        if all_shares_registered::<T>(self) {
            self.v1.status = SimpleCrowdfundingStatus::Inactive;
        }
        Ok(())
    }
}

fn all_shares_registered<T: crate::Config>(
    cf: &SimpleCrowdfundingV2<
        T::AccountId,
        T::Moment,
        FTokenId<T>,
        FTokenBalance<T>,
        TransactionCtxId<T::TransactionCtx>
    >
) -> bool
{
    (cf.registered_shares as usize) == cf.v1.shares.len()
}

impl<T: crate::Config> CrowdfundingT<T>
    for SimpleCrowdfunding<
        T::Moment,
        FTokenId<T>,
        FTokenBalance<T>,
        TransactionCtxId<T::TransactionCtx>
    >
{
    fn new(
        ctx: T::TransactionCtx,
        creator: T::AccountId,
        account: T::AccountId,
        external_id: InvestmentId,
        start_time: T::Moment,
        end_time: T::Moment,
        asset_id: FTokenId<T>,
        soft_cap: FTokenBalance<T>,
        hard_cap: FTokenBalance<T>,
        shares: Vec<FToken<T>>
    ) -> Self
    {
        SimpleCrowdfunding {
            created_ctx: ctx.id(),
            external_id,
            start_time,
            end_time,
            status: SimpleCrowdfundingStatus::Pending,
            asset_id,
            total_amount: Default::default(),
            soft_cap: SerializableAtLeast32BitUnsigned(soft_cap),
            hard_cap: SerializableAtLeast32BitUnsigned(hard_cap),
            shares: shares.into_iter().map(|x| Asset::new(*x.id(), *x.payload())).collect(),
        }
    }

    fn id(&self) -> &InvestmentId {
        &self.external_id
    }

    fn register_share(&mut self) -> Result<(), SimpleCrowdfundingStatus> {
        Err(self.status)
    }
}

pub trait CrowdfundingT<T: crate::Config>: Sized {
    fn new(
        ctx: T::TransactionCtx,
        creator: T::AccountId,
        account: T::AccountId,
        external_id: InvestmentId,
        start_time: T::Moment,
        end_time: T::Moment,
        asset_id: FTokenId<T>,
        soft_cap: FTokenBalance<T>,
        hard_cap: FTokenBalance<T>,
        shares: Vec<FToken<T>>,
    ) -> Self;

    fn id(&self) -> &InvestmentId;

    fn register_share(&mut self) -> Result<(), SimpleCrowdfundingStatus>;

    fn not_exist(id: InvestmentId) -> Result<(), crate::Error<T>> {
        Ok(ensure!(
            !SimpleCrowdfundingMapV2::<T>::contains_key(id),
            crate::Error::<T>::AlreadyExists
        ))
    }

    fn insert(cf: T::Crowdfunding) {
        SimpleCrowdfundingMapV2::<T>::insert(*cf.id(), cf);
    }

    fn find(id: InvestmentId) -> Result<T::Crowdfunding, crate::Error<T>> {
        SimpleCrowdfundingMapV2::<T>::try_get(id)
            .map_err(|_| crate::Error::NotFound)
    }
}

/// Unique InvestmentOpportunity ID reference
pub type InvestmentId = sp_core::H160;

#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, PartialOrd, Ord, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum SimpleCrowdfundingStatus {
    Active,
    Finished,
    Expired,
    Inactive,
    Pending
}

impl Default for SimpleCrowdfundingStatus {
    fn default() -> Self {
        SimpleCrowdfundingStatus::Inactive
    }
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum FundingModel<Moment, Asset> {
    SimpleCrowdfunding {
        /// a moment when the crowdfunding starts. Must be later than current moment.
        start_time: Moment,
        /// a moment when the crowdfunding ends. Must be later than `start_time`.
        end_time: Moment,
        /// amount of units to raise.
        soft_cap: Asset,
        /// amount upper limit of units to raise. Must be greater or equal to `soft_cap`.
        hard_cap: Asset,
    },
}

/// The object represents a sale of tokens with various parameters.
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct SimpleCrowdfundingV2<Account, Moment, AssetId, AssetBalance: Clone + AtLeast32BitUnsigned, CtxId> {
    v1: SimpleCrowdfunding<Moment, AssetId, AssetBalance, CtxId>,
    creator: Account,
    account: Account,
    registered_shares: u16,
}

/// The object represents a sale of tokens with various parameters.
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct SimpleCrowdfunding<Moment, AssetId, AssetBalance: Clone + AtLeast32BitUnsigned, CtxId> {
    // #[cfg_attr(feature = "std", serde(skip))]
    pub created_ctx: CtxId,
    /// Reference for external world and uniques control
    pub external_id: InvestmentId,
    /// When the sale starts
    pub start_time: Moment,
    /// When it supposed to end
    pub end_time: Moment,
    pub status: SimpleCrowdfundingStatus,
    pub asset_id: AssetId,
    /// How many contributions already reserved
    pub total_amount: SerializableAtLeast32BitUnsigned<AssetBalance>,
    pub soft_cap: SerializableAtLeast32BitUnsigned<AssetBalance>,
    pub hard_cap: SerializableAtLeast32BitUnsigned<AssetBalance>,
    /// How many and what tokens supposed to sale
    pub shares: Vec<Asset<AssetId, AssetBalance>>,
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct Contribution<AccountId, Balance, Moment> {
    pub sale_id: InvestmentId,
    pub owner: AccountId,
    pub amount: Balance,
    pub time: Moment,
}
