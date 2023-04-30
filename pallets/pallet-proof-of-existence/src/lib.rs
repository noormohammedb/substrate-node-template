#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	// #[pallet::genrate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}
	#[pallet::event]
	pub enum Event<T: Config> {
		//
	}
	#[pallet::error]
	pub enum Error<T> {
		//
	}
	#[pallet::storage]
	pub type MyDb<T> = StorageValue<_, u32>;
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//
	}
}
