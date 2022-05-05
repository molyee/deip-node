#![allow(type_alias_bounds)]

use sp_runtime::{traits::{AtLeast32BitUnsigned, Saturating, Zero}, SaturatedConversion, DispatchError};

use deip_serializable_u128::SerializableAtLeast32BitUnsigned;
use deip_transaction_ctx::{TransactionCtxT, PortalCtxT, TransactionCtxId};

use codec::{Encode, Decode};
#[cfg(feature = "std")]
use serde::{self, Serialize, Deserialize};
use frame_support::{ensure, RuntimeDebug};
use frame_support::dispatch::{DispatchResult, DispatchResultWithPostInfo};
use frame_support::log::{debug};
use frame_support::traits::{Get, fungibles::Inspect};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_std::prelude::*;
use crate::{Config, Error, Event, Call, Pallet};
use deip_asset_system::{DeipAssetSystem, ReserveError, UnreserveError};
pub use crate::crowdfunding::*;
pub use deip_asset_system::asset::*;
use crate::{SimpleCrowdfundingMapV1, InvestmentMapV1};
use crate::weights::WeightInfo;

pub type FTokenId<T: Config> = <T as Config>::AssetId;

pub type FTokenBalance<T: Config> = <T as Config>::AssetPayload;

pub type FToken<T: Config> = <T as Config>::Asset;

pub type FundingModelOf<T: Config> = FundingModel<T::Moment, FTokenBalance<T>>;

pub type SimpleCrowdfundingOf<T: Config> = SimpleCrowdfunding<
    T::Moment,
    FTokenId<T>,
    FTokenBalance<T>,
    TransactionCtxId<<T as Config>::TransactionCtx>,
>;

pub type Investment<T: Config> = Contribution<
    T::AccountId,
    FTokenBalance<T>,
    T::Moment
>;

use frame_support::traits::{Currency, ReservableCurrency, WithdrawReasons, ExistenceRequirement};

impl<T: Config> CrowdfundingAccount<T> for T {}

trait CrowdfundingAccount<T: Config> {

    fn _create_account(
        cf_owner: &T::AccountId,
        cf_id: &InvestmentId,
    ) -> Result<T::AccountId, DispatchError>
    {
        let reserved = T::Currency::withdraw(
            cf_owner,
            T::Currency::minimum_balance(),
            WithdrawReasons::RESERVE,
            ExistenceRequirement::AllowDeath,
        )?;
        let investment_account = investment_account::<T>(cf_id.as_bytes());
        T::Currency::resolve_creating(
            &investment_account,
            reserved
        );

        Ok(investment_account)
    }

    fn _destroy_account(
        cf_owner: &T::AccountId,
        cf_id: &InvestmentId
    )
    {
        let deposited =
            T::Currency::deposit_creating(
                &cf_owner,
                T::Currency::minimum_balance()
            );
        T::Currency::settle(
            &investment_account::<T>(cf_id.as_bytes()),
            deposited,
            WithdrawReasons::TRANSFER,
            ExistenceRequirement::AllowDeath,
        ).unwrap_or_else(|_| panic!("should be reserved in transactionally_reserve"));
    }
}

impl<T: Config + CrowdfundingAccount<T>> CrowdfundingCreate<T> for T {}

trait CrowdfundingCreate<T: Config>: CrowdfundingAccount<T> {
    fn _create_crowdfunding(
        creator: T::AccountId,
        external_id: InvestmentId,
        start_time: T::Moment,
        end_time: T::Moment,
        asset_to_raise: FTokenId<T>,
        soft_cap: FTokenBalance<T>,
        hard_cap: FTokenBalance<T>,
        shares: Vec<FToken<T>>,
    ) -> DispatchResult
    {
        let timestamp = pallet_timestamp::Pallet::<T>::get();
        ensure!(
            start_time >= timestamp,
            Error::<T>::StartTimeMustBeLaterOrEqualCurrentMoment
        );
        ensure!(
            end_time > start_time,
            Error::<T>::EndTimeMustBeLaterStartTime
        );

        ensure!(
            soft_cap > Zero::zero(),
            Error::<T>::SoftCapMustBeGreaterOrEqualMinimum
        );
        ensure!(
            hard_cap >= soft_cap,
            Error::<T>::HardCapShouldBeGreaterOrEqualSoftCap
        );

        ensure!(!shares.is_empty(), Error::<T>::SecurityTokenNotSpecified);
        for share in &shares {
            ensure!(share.id() != &asset_to_raise, Error::<T>::WrongAssetId);

            ensure!(
                share.payload() > &Zero::zero(),
                Error::<T>::AssetAmountMustBePositive
            );
        }

        ensure!(
            !SimpleCrowdfundingMapV1::<T>::contains_key(external_id),
            Error::<T>::AlreadyExists
        );

        let investment_account = Self::_create_account(
            &creator,
            &external_id
        ).map_err(|_| Error::<T>::BalanceIsNotEnough)?;

        for unit in shares.iter() {
            unit.transfer(
                creator.clone(),
                investment_account.clone(),
            );
            // match e {
            //     ReserveError::<FTokenId<T>>::NotEnoughBalance =>
            //         return Err(Error::<T>::BalanceIsNotEnough.into()),
            //     ReserveError::<FTokenId<T>>::AssetTransferFailed(_) =>
            //         return Err(Error::<T>::FailedToReserveAsset.into()),
            //     ReserveError::<FTokenId<T>>::AlreadyReserved =>
            //         return Err(Error::<T>::AlreadyExists.into()),
            // };
        }

        use crate::crowdfunding::CrowdfundingT;

        let crowdfunding = T::Crowdfunding::new(
            T::TransactionCtx::current(),
            creator,
            external_id,
            start_time,
            end_time,
            asset_to_raise,
            soft_cap,
            hard_cap,
            shares
        );
        T::Crowdfunding::insert(crowdfunding);

        Pallet::<T>::deposit_event(Event::<T>::SimpleCrowdfundingCreated(external_id));

        Ok(())
    }
}

impl<T: Config> Module<T> for T
    where T: CrowdfundingAccount<T>,
          T: CrowdfundingCreate<T>
{}

trait Module<T: Config>:
    CrowdfundingAccount<T> +
    CrowdfundingCreate<T>
{

    fn _share(
        from: &SimpleCrowdfundingOf<T>,
        to: &Investment<T>,
        unit: T::Asset,
    )
    {
        unit.transfer(
            investment_account::<T>(from.external_id.as_bytes()),
            to.owner.clone(),
        );
    }

    fn _purchase(
        from: T::AccountId,
        to: &SimpleCrowdfundingOf<T>,
        amount: FTokenBalance<T>,
    ) -> Result<(), UnreserveError<FTokenId<T>>>
    {
        T::Asset::new(to.asset_id, amount).transfer(
            from,
            investment_account::<T>(to.external_id.as_bytes())
        );
        Ok(())
    }

    // #[transactional]
    fn _abort(
        sale: &SimpleCrowdfundingOf<T>,
        sale_owner: &T::AccountId,
        // shares: &[(DeipAssetId<T>, DeipAssetBalance<T>)],
    ) -> Result<(), UnreserveError<FTokenId<T>>>
    {
        let sale_account = investment_account::<T>(sale.external_id.as_bytes());

        if let Ok(ref c) = InvestmentMapV1::<T>::try_get(sale.external_id) {
            for (_, ref investment) in c {

                T::Asset::new(sale.asset_id, investment.amount).transfer(
                    sale_account.clone(),
                    investment.owner.clone(),
                );

                frame_system::Pallet::<T>::dec_consumers(&investment.owner);
            }
            InvestmentMapV1::<T>::remove(sale.external_id);
        }

        //////////////////

        for asset_id in sale.shares.iter().map(|x| x.id()) {

            let total = T::Asset::balance(*asset_id, &sale_account);
            if total.payload().is_zero() {
                continue
            }

            total.transfer(
                sale_account.clone(),
                sale_owner.clone(),
            );
            // if result.is_err() {
            //     return Err(UnreserveError::AssetTransferFailed(*asset_id))
            // }
        }

        Self::_destroy_account(sale_owner, &sale.external_id);

        Ok(())
    }

    // #[transactional]
    fn _raise(
        sale: &SimpleCrowdfundingOf<T>,
        sale_owner: T::AccountId,
        // shares: &[(DeipAssetId<T>, DeipAssetBalance<T>)],
        // asset_to_raise: DeipAssetId<T>,
    ) -> Result<(), UnreserveError<FTokenId<T>>>
    {
        let deposited =
            T::Currency::deposit_creating(&sale_owner, T::Currency::minimum_balance());

        let sale_account = investment_account::<T>(sale.external_id.as_bytes());

        let total = T::Asset::balance(sale.asset_id, &sale_account);
        if total.payload().is_zero() {
            return Ok(())
        }

        total.transfer(
            sale_account.clone(),
            sale_owner.clone(),
        );
        // if result.is_err() {
        //     return Err(UnreserveError::AssetTransferFailed(*asset_id))
        // }

        T::Currency::settle(
            &sale_account,
            deposited,
            WithdrawReasons::TRANSFER,
            ExistenceRequirement::AllowDeath,
        )
        .unwrap_or_else(|_| panic!("should be reserved in transactionally_reserve"));

        Ok(())
    }
}

pub struct Reserve {}

impl<T: Config> Pallet<T> {
    pub(super) fn create_investment_opportunity_impl(
        external_id: InvestmentId,
        creator: T::AccountId,
        shares: Vec<FToken<T>>,
        funding_model: FundingModelOf<T>,
    ) -> DispatchResult {
        ensure!(
            shares.len() <= T::MaxInvestmentShares::get() as usize,
            Error::<T>::TooMuchShares
        );

        match funding_model {
            FundingModel::SimpleCrowdfunding { start_time, end_time, soft_cap, hard_cap } => {
                let asset_to_raise = Default::default();
                T::_create_crowdfunding(
                    creator,
                    external_id,
                    start_time,
                    end_time,
                    asset_to_raise,
                    soft_cap,
                    hard_cap,
                    shares,
                )
            },
        }
    }

    pub(super) fn activate_crowdfunding_impl(sale_id: InvestmentId) -> DispatchResult {
        SimpleCrowdfundingMapV1::<T>::mutate_exists(sale_id, |maybe_sale| -> DispatchResult {
            let sale = match maybe_sale.as_mut() {
                None => return Err(Error::<T>::NotFound.into()),
                Some(s) => s,
            };

            match sale.status {
                SimpleCrowdfundingStatus::Active => return Ok(()),
                SimpleCrowdfundingStatus::Inactive => ensure!(
                    pallet_timestamp::Pallet::<T>::get() >= sale.start_time,
                    Error::<T>::ShouldBeStarted
                ),
                _ => return Err(Error::<T>::ShouldBeInactive.into()),
            };

            sale.status = SimpleCrowdfundingStatus::Active;
            Self::deposit_event(Event::SimpleCrowdfundingActivated(sale_id));

            Ok(())
        })
    }

    pub(super) fn expire_crowdfunding_impl(sale_id: InvestmentId) -> DispatchResultWithPostInfo
    {
        let mut sale
            = SimpleCrowdfundingMapV1::<T>::get(sale_id)
            .ok_or(Error::<T>::NotFound)?;

        match sale.status {
            SimpleCrowdfundingStatus::Expired => {
                let weight = T::DeipInvestmentWeightInfo::expire_crowdfunding_already_expired();
                return Ok(Some(weight).into())
            },
            SimpleCrowdfundingStatus::Active => ensure!(
                pallet_timestamp::Pallet::<T>::get() >= sale.end_time,
                Error::<T>::ExpirationWrongState
            ),
            _ => return Err(Error::<T>::ShouldBeActive)?,
        };

        sale.status = SimpleCrowdfundingStatus::Expired;
        T::_abort(
            &sale,
            // sale_owner,
            &Default::default()
        ).unwrap_or_else(|_| panic!("assets should be reserved earlier"));
        Self::deposit_event(Event::SimpleCrowdfundingExpired(sale.external_id));
        SimpleCrowdfundingMapV1::<T>::insert(sale_id, sale);

        Ok(None.into())
    }

    pub(super) fn finish_crowdfunding_impl(sale_id: InvestmentId) -> DispatchResult
    {
        let mut sale
            = SimpleCrowdfundingMapV1::<T>::get(sale_id)
            .ok_or(Error::<T>::NotFound)?;

        match sale.status {
            SimpleCrowdfundingStatus::Finished => return Ok(()),
            SimpleCrowdfundingStatus::Active => (),
            _ => return Err(Error::<T>::ShouldBeActive.into()),
        };

        sale.status = SimpleCrowdfundingStatus::Finished;

        Self::finish(&sale);
        SimpleCrowdfundingMapV1::<T>::insert(sale_id, sale);

        Ok(())
    }

    pub(super) fn process_investment_opportunities_offchain() {
        let now = pallet_timestamp::Pallet::<T>::get();
        for (id, sale) in SimpleCrowdfundingMapV1::<T>::iter() {
            if sale.end_time <= now && matches!(sale.status, SimpleCrowdfundingStatus::Active) {
                if sale.total_amount.0 < sale.soft_cap.0 {
                    let call = Call::<T>::expire_crowdfunding { sale_id: id };
                    let submit = T::TransactionCtx::submit_postponed(call.into(), sale.created_ctx);

                    debug!("submit expire_crowdfunding: {}", submit.is_ok());
                } else if sale.total_amount.0 >= sale.soft_cap.0 {
                    let call = Call::<T>::finish_crowdfunding { sale_id: id };
                    let submit = T::TransactionCtx::submit_postponed(call.into(), sale.created_ctx);
                    debug!("submit finish_crowdfunding: {}", submit.is_ok());
                }
            } else if sale.end_time > now {
                if now >= sale.start_time && matches!(sale.status, SimpleCrowdfundingStatus::Inactive) {
                    let call = Call::<T>::activate_crowdfunding { sale_id: id };
                    let submit = T::TransactionCtx::submit_postponed(call.into(), sale.created_ctx);
                    debug!("submit activate_crowdfunding: {}", submit.is_ok());
                }
            }
        }
    }

    fn finish(sale: &SimpleCrowdfundingOf<T>) {
        let investments = InvestmentMapV1::<T>::try_get(sale.external_id)
            .expect("about to finish, but there are no contributions?");

        if investments.is_empty() {
            panic!("about to finish, but there are no contributors?")
        }

        let contribution = ContributionAccept::<T>::new(sale);

        for share in &sale.shares {
            let mut share_remains = share.amount().clone();

            for (_, ref investment) in investments.iter() {

                share_remains = contribution.accept(
                    investment,
                    &<FToken<T>>::new(*share.id(), *share.amount()),
                    share_remains
                );
            }
        }

        T::_raise(
            sale,
            // sale_owner
            Default::default()
        ).unwrap_or_else(|_| panic!("remaining assets should be reserved earlier"));

        InvestmentMapV1::<T>::remove(sale.external_id);

        Self::deposit_event(Event::SimpleCrowdfundingFinished(sale.external_id));
    }

    pub(super) fn invest_to_crowdfunding_impl(
        account: T::AccountId,
        sale_id: InvestmentId,
        asset: FToken<T>,
    ) -> DispatchResultWithPostInfo {
        let sale = SimpleCrowdfundingMapV1::<T>::try_get(sale_id)
            .map_err(|_| Error::<T>::InvestingNotFound)?;

        ensure!(
            matches!(sale.status, SimpleCrowdfundingStatus::Active),
            Error::<T>::InvestingNotActive
        );

        ensure!(sale.asset_id == *asset.id(), Error::<T>::InvestingWrongAsset);

        fn hard_cap_overflows<T: Config>(
            sale: &SimpleCrowdfundingOf<T>,
            amount: T::AssetPayload
        ) -> bool
        {
            sale.total_amount.0.saturating_add(amount) >= sale.hard_cap.0
        }

        fn correct_hard_cap<T: Config>(
            sale: &SimpleCrowdfundingOf<T>,
            amount: T::AssetPayload,
        ) -> T::AssetPayload
        {
            if hard_cap_overflows::<T>(sale, amount) {
                sale.hard_cap.0.saturating_sub(sale.total_amount.0)
            } else {
                amount
            }
        }

        let hard_cap_reached = hard_cap_overflows::<T>(&sale, *asset.payload());
        let amount_to_contribute = correct_hard_cap::<T>(&sale, *asset.payload());

        ensure!(
            // T::transfer_to_reserved(&account, sale.external_id, amount_to_contribute).is_ok(),
            T::_purchase(account.clone(), &sale, amount_to_contribute).is_ok(),
            Error::<T>::InvestingNotEnoughFunds
        );

        InvestmentMapV1::<T>::mutate_exists(sale_id, |contributions| {
            let mut_contributions = match contributions.as_mut() {
                None => {
                    // If the account executes the extrinsic then it exists, so it should have at least one provider
                    // so this cannot fail... but being defensive anyway.
                    let _ = frame_system::Pallet::<T>::inc_consumers(&account);

                    *contributions = Some(vec![(
                        account.clone(),
                        Contribution {
                            sale_id,
                            owner: account.clone(),
                            amount: amount_to_contribute,
                            time: pallet_timestamp::Pallet::<T>::get(),
                        },
                    )]);
                    return
                },
                Some(c) => c,
            };

            match mut_contributions.binary_search_by_key(&&account, |&(ref a, _)| a) {
                Err(i) => {
                    // see comment above
                    let _ = frame_system::Pallet::<T>::inc_consumers(&account);

                    mut_contributions.insert(
                        i,
                        (
                            account.clone(),
                            Contribution {
                                sale_id,
                                owner: account.clone(),
                                amount: amount_to_contribute,
                                time: pallet_timestamp::Pallet::<T>::get(),
                            },
                        ),
                    );
                },
                Ok(i) => {
                    mut_contributions[i].1.amount =
                        amount_to_contribute.saturating_add(mut_contributions[i].1.amount);
                },
            };
        });

        // Self::collect_funds(sale_id, amount_to_contribute).expect("collect; already found");
        let _ = SimpleCrowdfundingMapV1::<T>::mutate_exists(sale_id, |sale| -> Result<(), ()> {
            match sale.as_mut() {
                Some(s) => s.total_amount.0 = amount_to_contribute.saturating_add(s.total_amount.0),
                None => panic!("collect; already found"),
            }
            Ok(())
        });

        Self::deposit_event(Event::<T>::Invested(sale_id, account.clone()));

        if hard_cap_reached {
            // Self::finish_crowdfunding_by_id(sale_id).expect("finish; already found");
            let _ = match SimpleCrowdfundingMapV1::<T>::try_get(sale_id) {
                Err(_) => panic!("finish; already found"),
                Ok(sale) => {
                    // Self::update_status(&sale, SimpleCrowdfundingStatus::Finished);
                    SimpleCrowdfundingMapV1::<T>::mutate_exists(sale.external_id, |maybe_sale| -> () {
                        let sale = maybe_sale.as_mut().expect("we keep collections in sync");
                        sale.status = SimpleCrowdfundingStatus::Finished;
                    });
                    Self::finish(&sale);
                    // Self::deposit_event(Event::<T>::HardCapReached(sale_id, account.clone()));
                    Result::<_, ()>::Ok(())
                },
            };
            return Ok(Some(T::DeipInvestmentWeightInfo::invest_hard_cap_reached()).into())
        }

        Ok(Some(T::DeipInvestmentWeightInfo::invest()).into())
    }
}

pub trait ContributionT<T: Config>: ContributionAcceptT<T> {}

impl<T: Config, U> ContributionT<T> for U
    where U: ContributionAcceptT<T>
{}

pub trait ContributionAcceptT<T: Config> {
    fn accept(
        &self,
        investment: &Investment<T>,
        share: &FToken<T>,
        share_remaining: FTokenBalance<T>,
    ) -> ShareRemaining<T>;
}
pub type ShareRemaining<T> = FTokenBalance<T>;
impl<T: Config> ContributionAcceptT<T> for ContributionAccept<'_, T> {
    fn accept(
        &self,
        investment: &Investment<T>,
        share: &FToken<T>,
        share_remains: FTokenBalance<T>,
    ) -> ShareRemaining<T>
    {
        frame_system::Pallet::<T>::dec_consumers(&investment.owner);

        if share_remains.is_zero() {
            // Why it can be a zero ?
            return share_remains
        }

        let token_amount: FTokenBalance<T>
            = self.token_amount(investment, share)
            .calc()
            .saturated_into();

        if token_amount.is_zero() {
            // Why it can be a zero ?
            return share_remains
        }

        use deip_asset_system::{asset::{GenericAssetT}};

        T::_share(
            &self.sale,
            investment,
            T::Asset::new(*share.id(), token_amount)
        );

        share_remains - token_amount
    }
}

pub fn investment_account<T: Config>(id: &[u8]) -> T::AccountId {
    let entropy =
        (b"deip/investments/", id).using_encoded(sp_io::hashing::blake2_256);
    T::AccountId::decode(&mut &entropy[..]).unwrap_or_default()
}

impl<'a, T: Config> ContributionAccept<'a, T> {
    pub fn new(sale: &'a SimpleCrowdfundingOf<T>) -> Self
    {
        Self { sale }
    }
    pub fn token_amount(
        &self,
        investment: &Investment<T>,
        share: &FToken<T>,
    ) -> TokenAmount
    {
        let investment_amount = investment.amount.saturated_into::<u128>();
        let share_amount = (*share.payload()).saturated_into::<u128>();
        let sale_amount = self.sale.total_amount.0.saturated_into::<u128>();
        TokenAmount {
            investment_amount,
            share_amount,
            sale_amount
        }
    }
}
impl TokenAmount {
    pub fn calc(&self) -> u128
    {
        if self.sale_amount.is_zero() { return 0 }
        // similar to frame_support::traits::Imbalance::ration
        // [ investment_amount / x = sale_amount / share_amount ]
        // [ x = investment_amount * share_amount / sale_amount ]
        self.investment_amount
            .saturating_mul(self.share_amount)
            / self.sale_amount
    }
}
pub struct ContributionAccept<'a, T: Config> {
    sale: &'a SimpleCrowdfundingOf<T>,
}
pub struct TokenAmount {
    investment_amount: u128,
    share_amount: u128,
    sale_amount: u128
}
