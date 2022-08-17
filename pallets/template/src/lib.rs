#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config] // <-- Step 2. code block will replace this.
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}
	#[pallet::event] // <-- Step 3. code block will replace this.
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated { who: T::AccountId, claim: T::Hash },
		ClaimRevoked { who: T::AccountId, claim: T::Hash },
	}

	#[pallet::error] // <-- Step 4. code block will replace this.
	pub enum Error<T> {
		AlreadyClaimed,
		NoSuchClaim,
		NotClaimOwner,
	}

	#[pallet::storage] // <-- Step 5. code block will replace this.
	pub(super) type Claims<T: Config> =
		StorageMap<_, Blake2_128Concat, T::Hash, (T::AccountId, T::BlockNumber)>;

	#[pallet::call] // <-- Step 6. code block will replace this.
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_claim(origin: OriginFor<T>, claim: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(!Claims::<T>::contains_key(&claim), Error::<T>::AlreadyClaimed);
			let current_block = <frame_system::Pallet<T>>::block_number();

			Claims::<T>::insert(claim, (&sender, current_block));

			Ok(())
		}
	}
}
