#![cfg_attr(not(feature = "std"), no_std)]

//! A pallet that implements a storage set on top of a sorted vec and demonstrates performance
//! tradeoffs when using map sets.

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure};
use frame_system::{self as system, ensure_signed};
use sp_std::prelude::*;
use balances_s::Traitz;
use parity_scale_codec::{Codec, Encode, Decode, FullCodec, FullEncode, EncodeLike};


// #[cfg(test)]
// mod tests;

/// A maximum number of members. When membership reaches this number, no new members may join.
// pub const MAX_MEMBERS: usize = 16;

pub trait Trait: system::Trait + balances_s::Traitz<AccountIdz, Balancez> {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type AccountIdz: Parameter + Member + MaybeSerializeDeserialize + Debug + MaybeDisplay + Ord + Default;
	type Balancez: Parameter + Member + AtLeast32BitUnsigned + Codec + Default + Copy + MaybeSerializeDeserialize + Debug;
}

// decl_storage! {
// 	trait Store for Module<T: Trait> as VecSet {
// 		// The set of all members. Stored as a single vec
// 		Members get(fn members): Vec<T::AccountId>;
// 	}
// } 

decl_storage! {
	trait Store for Module<T: Trait> as Token {
		pub Balances get(fn get_balance): map hasher(blake2_128_concat) <T as balances_s::Traitz>::AccountIdz => <T as balances_s::Traitz>::Balancez;

		pub TotalSupply get(fn total_supply): u64 = 21000000;

		Init get(fn is_init): bool;
	}
}

// decl_event!(
// 	pub enum Event<T>
// 	where
// 		AccountId = <T as system::Trait>::AccountId,
// 	{
// 		/// Added a member
// 		MemberAdded(AccountId),
// 		/// Removed a member
// 		MemberRemoved(AccountId),
// 	}
// );

decl_event!(
	pub enum Event<T>
	where
		AccountIdz = <T as balances_s::Traitz>::AccountIdz,
		Balancez = <T as balances_s::Traitz>::Balancez,
	{
		/// Token was initialized by user
		Initialized(AccountIdz),
		/// Tokens successfully transferred between users
		Transfer(AccountIdz, AccountIdz, Balancez), // (from, to, value)
	}
);

// decl_error! {
// 	pub enum Error for Module<T: Trait> {
// 		/// Cannot join as a member because you are already a member
// 		AlreadyMember,
// 		/// Cannot give up membership because you are not currently a member
// 		NotMember,
// 		/// Cannot add another member because the limit is already reached
// 		MembershipLimitReached,
// 	}
// }

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Attempted to initialize the token after it had already been initialized.
		AlreadyInitialized,
		/// Attempted to transfer more funds than were available
		InsufficientFunds,
	}
}

// decl_module! {
// 	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
// 		fn deposit_event() = default;

// 		// type Error = Error<T>;

// 		/// Adds a member to the membership set unless the max is reached
// 		#[weight = 10_000]
// 		pub fn add_member(origin) -> DispatchResult {
// 			let new_member = ensure_signed(origin)?;

// 			let mut members = Members::<T>::get();
// 			ensure!(members.len() < MAX_MEMBERS, Error::<T>::MembershipLimitReached);

// 			// We don't want to add duplicate members, so we check whether the potential new
// 			// member is already present in the list. Because the list is always ordered, we can
// 			// leverage the binary search which makes this check O(log n).
// 			match members.binary_search(&new_member) {
// 				// If the search succeeds, the caller is already a member, so just return
// 				Ok(_) => Err(Error::<T>::AlreadyMember.into()),
// 				// If the search fails, the caller is not a member and we learned the index where
// 				// they should be inserted
// 				Err(index) => {
// 					members.insert(index, new_member.clone());
// 					Members::<T>::put(members);
// 					Self::deposit_event(RawEvent::MemberAdded(new_member));
// 					Ok(())
// 				}
// 			}
// 		}

// 		/// Removes a member.
// 		#[weight = 10_000]
// 		fn remove_member(origin) -> DispatchResult {
// 			let old_member = ensure_signed(origin)?;

// 			let mut members = Members::<T>::get();

// 			// We have to find out if the member exists in the sorted vec, and, if so, where.
// 			match members.binary_search(&old_member) {
// 				// If the search succeeds, the caller is a member, so remove her
// 				Ok(index) => {
// 					members.remove(index);
// 					Members::<T>::put(members);
// 					Self::deposit_event(RawEvent::MemberRemoved(old_member));
// 					Ok(())
// 				},
// 				// If the search fails, the caller is not a member, so just return
// 				Err(_) => Err(Error::<T>::NotMember.into()),
// 			}
// 		}

// 		// also see `append_or_insert`, `append_or_put` in pallet-elections/phragmen, democracy
// 	}
// }

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// Initialize the token
		/// transfers the total_supply amout to the caller
		#[weight = 10_000]
		fn init(origin) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(!Self::is_init(), <Error<T>>::AlreadyInitialized);

			<Balances<T>>::insert(sender, Self::total_supply());

			Init::put(true);
			Ok(())
		}

		/// Transfer tokens from one account to another
		#[weight = 10_000]
		fn transfer(_origin, to: <T as balances_s::Traitz>::AccountIdz, value: <T as balances_s::Traitz>::Balancez) -> DispatchResult {
			let sender = ensure_signed(_origin)?;
			let sender_balance = Self::get_balance(&sender);
			let receiver_balance = Self::get_balance(&to);

			// Calculate new balances
			let updated_from_balance = sender_balance.checked_sub(value).ok_or(<Error<T>>::InsufficientFunds)?;
			let updated_to_balance = receiver_balance.checked_add(value).expect("Entire supply fits in u64; qed");

			// Write new balances to storage
			<Balances<T>>::insert(&sender, updated_from_balance);
			<Balances<T>>::insert(&to, updated_to_balance);

			// Need to write custom function in Traitz to replace `RawEvent::Transfer`
			Self::deposit_event(RawEvent::Transfer(sender, to, value));
			Ok(())
		}
	}
}

impl<T: Trait> Traitz for Module<T> {
	type AccountIdz = <T as balances_s::Traitz>::AccountIdz;
	type Balancez = <T as balances_s::Traitz>::Balancez;

	 fn bals() -> u64 {
		Self::total_supply()
	}

	fn insertz<KeyArg: EncodeLike<K>, ValArg: EncodeLike<V>>(key: KeyArg, val: ValArg) {
		
	}
}
