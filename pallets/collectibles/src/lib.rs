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
		NotCollectible,
		NotOwner,
		TransferToSelf,
		BidPriceTooLow,
		NotForSale,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CollectibleCreated {
			collectible: [u8; 16],
			owner: T::AccountId,
		},
		TransferSucceeded {
			from: T::AccountId,
			to: T::AccountId,
			collectible: [u8; 16],
		},
		PriceSet {
			collectible: [u8; 16],
			price: Option<BalanceOf<T>>,
		},
		Sold {
			seller: T::AccountId,
			buyer: T::AccountId,
			collectible: [u8; 16],
			price: BalanceOf<T>,
		},
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

		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			unique_id: [u8; 16],
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			let collectible =
				CollectibleMap::<T>::get(&unique_id).ok_or(Error::<T>::NotCollectible)?;
			ensure!(collectible.owner == from, Error::<T>::NotOwner);
			Self::do_transfer(unique_id, to)?;
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn set_price(
			origin: OriginFor<T>,
			unique_id: [u8; 16],
			price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			let mut collectible =
				CollectibleMap::<T>::get(unique_id).ok_or(Error::<T>::NotCollectible)?;
			ensure!(collectible.owner == caller, Error::<T>::NotOwner);

			collectible.price = price;
			CollectibleMap::<T>::insert(&unique_id, collectible);

			Self::deposit_event(Event::PriceSet { collectible: unique_id, price });
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn buy_collectible(
			origin: OriginFor<T>,
			unique_id: [u8; 16],
			bid_price: BalanceOf<T>,
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;
			Self::do_buy_collectible(unique_id, buyer, bid_price)?;

			Ok(())
		}
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

		pub fn do_transfer(collectible_id: [u8; 16], to: T::AccountId) -> DispatchResult {
			let mut collectible =
				CollectibleMap::<T>::get(&collectible_id).ok_or(Error::<T>::NotCollectible)?;

			let from = collectible.owner;

			ensure!(from != to, Error::<T>::TransferToSelf);

			let mut from_owner = OwnerOfCollectibles::<T>::get(&from);

			if let Some(ind) = from_owner.iter().position(|&id| id == collectible_id) {
				from_owner.swap_remove(ind);
			} else {
				return Err(Error::<T>::NotCollectible.into())
			}

			let mut to_owned = OwnerOfCollectibles::<T>::get(&to);
			to_owned
				.try_push(collectible_id)
				.map_err(|_id| Error::<T>::MaximumCollectiblesOwned)?;
			collectible.owner = to.clone();
			collectible.price = None;
			CollectibleMap::<T>::insert(&collectible_id, collectible);
			OwnerOfCollectibles::<T>::insert(&to, to_owned);
			OwnerOfCollectibles::<T>::insert(&from, from_owner);

			Self::deposit_event(Event::TransferSucceeded { from, to, collectible: collectible_id });

			Ok(())
		}

		pub fn do_buy_collectible(
			unique_id: [u8; 16],
			to: T::AccountId,
			bid_price: BalanceOf<T>,
		) -> DispatchResult {
			let collectible =
				CollectibleMap::<T>::get(unique_id).ok_or(Error::<T>::NotCollectible)?;
			if let (Some(price), from) = (collectible.price, collectible.owner) {
				ensure!(bid_price >= price, Error::<T>::BidPriceTooLow);
				Self::do_transfer(unique_id, to.clone())?;
				T::Currency::transfer(
					&to,
					&from,
					price,
					frame_support::traits::ExistenceRequirement::KeepAlive,
				)?;

				Self::deposit_event(Event::Sold {
					seller: from.clone(),
					buyer: to.clone(),
					collectible: unique_id,
					price,
				});
			} else {
				return Err(Error::<T>::NotForSale.into())
			}
			Ok(())
		}
	}
}
