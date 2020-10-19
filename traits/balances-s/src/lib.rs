#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_std::{cmp, result, mem, fmt::Debug, ops::BitOr, convert::Infallible};
use codec::{Codec, Encode, Decode, FullCodec, FullEncode, EncodeLike};
use frame_support::{
	StorageValue, Parameter, decl_event, decl_storage, decl_module, decl_error, ensure,
	weights::Weight,
	traits::{
		Currency, OnKilledAccount, OnUnbalanced, TryDrop, StoredMap,
		WithdrawReason, WithdrawReasons, LockIdentifier, LockableCurrency, ExistenceRequirement,
		Imbalance, SignedImbalance, ReservableCurrency, Get, ExistenceRequirement::KeepAlive,
		ExistenceRequirement::AllowDeath, IsDeadAccount, BalanceStatus as Status,
	}
};
use sp_runtime::{
	RuntimeDebug, DispatchResult, DispatchError,
	traits::{
		Zero, AtLeast32BitUnsigned, StaticLookup, Member, CheckedAdd, CheckedSub,
		MaybeSerializeDeserialize, Saturating, Bounded, MaybeDisplay
	},
};
// pub use self::imbalances::{PositiveImbalance, NegativeImbalance};


/// Types that implement the Traitz trait are able to supply an alternate balance module
pub trait Traitz<K: FullEncode, V: FullCodec> { // any object must have these associated types
	type AccountIdz: Parameter + Member + MaybeSerializeDeserialize + Debug + MaybeDisplay + Ord + Default;
	type Balancez: Parameter + Member + AtLeast32BitUnsigned + Codec + Default + Copy + MaybeSerializeDeserialize + Debug;

	fn bals() -> u64;

	fn insertz<KeyArg: EncodeLike<K>, ValArg: EncodeLike<V>>(
		key: KeyArg,
		val: ValArg
	);
}
