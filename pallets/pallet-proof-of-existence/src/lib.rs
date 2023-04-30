#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	// #[pallet::generate_store(pub(super) trait Store)]	// deprecated
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated { who: T::AccountId, claim: T::Hash },
		ClaimRevoked { who: T::AccountId, claim: T::Hash },
	}
	#[pallet::error]
	pub enum Error<T> {
		AlreadyClaimed,
		NoSuchClaim,
		NotClaimOwner,
	}
	#[pallet::storage]
	pub(super) type Claims<T: Config> =
		StorageMap<_, Blake2_128Concat, T::Hash, (T::AccountId, T::BlockNumber)>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		#[pallet::call_index(1)]
		pub fn create_claim(origin: OriginFor<T>, claim: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// let foo = Claims::<T>::get(&claim).ok_or("Foo");
			// let foo = Claims::<T>::contains_key(&claim);
			ensure!(!Claims::<T>::contains_key(&claim), Error::<T>::AlreadyClaimed);

			let current_block = <frame_system::Pallet<T>>::block_number();

			Claims::<T>::insert(&claim, (&sender, current_block));

			Self::deposit_event(Event::ClaimCreated { who: sender, claim });

			Ok(())
		}

		#[pallet::weight(0)]
		#[pallet::call_index(2)]
		pub fn revoke_claim(origin: OriginFor<T>, claim: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			/*
			ensure!(Claims::<T>::contains_key(&claim), Error::<T>::NoSuchClaim);

			let foo = Claims::<T>::get(&claim);
			if let Some(data) = foo {
				ensure!(data.0 == sender, Error::<T>::NotClaimOwner);
				Claims::<T>::remove(&claim);
				Self::deposit_event(Event::<T>::ClaimRevoked { who: sender, claim });
				Ok(())
			} else {
				fail!(Error::<T>::NoSuchClaim)
			}
			*/

			let (owner, _) = Claims::<T>::get(&claim).ok_or(Error::<T>::NoSuchClaim)?;

			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			Claims::<T>::remove(&claim);

			Self::deposit_event(Event::<T>::ClaimRevoked { who: sender, claim });
			Ok(())
		}
	}
}
