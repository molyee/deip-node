//! # DEIP Council Module
//! A module to make transactions with a Portal signature
//!
//! - [`Config`](./trait.Config.html)
//! - [`Call`](./enum.Call.html)
//!
//! ## Overview
//! A module to make transactions with a Portal signature
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `create` - Create a Portal.
//! * `update` - Update a Portal.
//! * `schedule` - Schedule an extrinsic to be executed within Portal context.
//! * `exec` - Call-wrapper that may be scheduled.
//!
//! [`Call`]: ./enum.Call.html
//! [`Config`]: ./trait.Config.html

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_variables)]

#[doc(inline)]
pub use pallet::*;
#[cfg(test)]
mod tests;
pub mod benchmarking;
pub mod weights;

#[frame_support::pallet]
#[doc(hidden)]
pub(crate) mod pallet {
    use crate::weights::WeightInfo;
    use super::*;
    use frame_system::pallet_prelude::*;
    use frame_system::RawOrigin;
    use frame_support::dispatch::DispatchResult;
    use frame_support::weights::{GetDispatchInfo, PostDispatchInfo};
    use frame_support::traits::StorageVersion;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::{IsSubType, UnfilteredDispatchable};
    use sp_std::prelude::*;
    use sp_std::fmt::Debug;
    use sp_runtime::traits::Dispatchable;

    /// Configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config + Debug + TypeInfo {
        /// Type represents events
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Type represents particular call from batch-transaction
        type Call: Parameter
            + Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
            + GetDispatchInfo
            + From<frame_system::pallet::Call<Self>>
            + From<crate::Call<Self>>
            + UnfilteredDispatchable<Origin = Self::Origin>
            + frame_support::dispatch::Codec
            + IsSubType<Call<Self>>
            + TypeInfo;

        type Weights: WeightInfo;
    }

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[doc(hidden)]
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        AlreadyInitialized,
        Uninitialized,
        PermissionDenied,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        CouncilInitialized(T::AccountId),
    }

    #[pallet::storage]
	#[pallet::getter(fn key)]
	pub(super) type Key<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight((T::Weights::init(), DispatchClass::Normal, Pays::Yes))]
        pub fn init(origin: OriginFor<T>, key: T::AccountId) -> DispatchResult {
            // TODO
            ensure_root(origin)?;
            ensure!(Self::key().is_none(), Error::<T>::AlreadyInitialized);
            //let dao = Self::load_dao(LoadBy::DaoId { id: &dao_id, who: &origin })
            Self::deposit_event(Event::CouncilInitialized(key));
            Ok(())
        }

        #[pallet::weight((T::Weights::update_runtime(code.len()), DispatchClass::Operational))]
        pub fn update_runtime(origin: OriginFor<T>, code: Vec<u8>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(Self::key().is_none(), Error::<T>::Uninitialized);
            ensure!(sender == Self::key().unwrap(), Error::<T>::PermissionDenied);
            let call = frame_system::Call::<T>::set_code { code };
            call.dispatch_bypass_filter(RawOrigin::Root.into())
        }
    }
}
