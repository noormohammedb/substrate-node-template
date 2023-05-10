#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	use frame_support::traits::{Currency, Randomness};

	#[pallet::pallet]
	// #[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config + scale_info::TypeInfo {
		type Currency: Currency<Self::AccountId>;
		type CollectionRandomness: Randomness<Self::Hash, Self::BlockNumber>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type MaximumOwned: Get<u32>;
	}

	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum Color {
		Red,
		Yellow,
		Blue,
		Green,
	}

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct Collectible<T: Config> {
		pub unique_id: [u8; 16],
		pub price: Option<BalanceOf<T>>,
		pub color: Color,
		pub owner: T::AccountId,
	}

	#[pallet::storage]
	pub(super) type CollectiblesCount<T: Config> = StorageValue<_, u64, ValueQuery>;
	#[pallet::storage]
	pub(super) type CollectibleMap<T: Config> =
		StorageMap<_, Twox64Concat, [u8; 16], Collectible<T>>;
	#[pallet::storage]
	// #[scale_info(skip_type_params(T))]
	pub(super) type OwnerOfCollectibles<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedVec<[u8; 16], T::MaximumOwned>,
		ValueQuery,
	>;

	#[pallet::error]
	pub enum Error<T> {
		DuplicateCollectible,
		MaximumCollectiblesOwned,
		BoundsOverflow,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CollectibleCreated { collectible: [u8; 16], owner: T::AccountId },
	}

	impl<T: Config> Pallet<T> {
		fn gen_unique_id() -> ([u8; 16], Color) {
			let random = T::CollectionRandomness::random(&b"unique_id"[..]).0;

			let unique_payload = (
				random,
				frame_system::Pallet::<T>::extrinsic_index().unwrap_or_default(),
				frame_system::Pallet::<T>::block_number(),
			);

			let encoded_payload = unique_payload.encode();
			let hash = frame_support::Hashable::blake2_128(&encoded_payload);

			if hash[0] % 2 == 0 {
				(hash, Color::Red)
			} else {
				(hash, Color::Yellow)
			}
		}

		//
		pub fn mint(
			owner: &T::AccountId,
			unique_id: [u8; 16],
			color: Color,
		) -> Result<[u8; 16], DispatchError> {
			let collectible =
				Collectible::<T> { unique_id, color, owner: owner.clone(), price: None };

			ensure!(
				!CollectibleMap::<T>::contains_key(&collectible.unique_id),
				Error::<T>::DuplicateCollectible
			);

			let count = CollectiblesCount::<T>::get();
			let new_count = count.checked_add(1).ok_or(Error::<T>::BoundsOverflow)?;

			OwnerOfCollectibles::<T>::try_append(&owner, collectible.unique_id)
				.map_err(|_| Error::<T>::MaximumCollectiblesOwned)?;

			CollectibleMap::<T>::insert(collectible.unique_id, collectible);
			CollectiblesCount::<T>::put(new_count);

			Self::deposit_event(Event::CollectibleCreated {
				collectible: unique_id,
				owner: owner.clone(),
			});

			// Ok([0u8; 16])
			Ok(unique_id)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_collectible(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let (collectible_gen_unique_id, color) = Self::gen_unique_id();

			Self::mint(&sender, collectible_gen_unique_id, color)?;

			Ok(())
		}
	}
}
