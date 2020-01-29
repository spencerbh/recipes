//! A dead simple runtime that has a single boolean storage value and three transactions. The transactions
//! available are Set, Clear, and Toggle.

// Some open questions:
// * How do I use storage? Can I do decl_storage! here like I would in a pallet?
// * What are core apis (eg. initialize_block, execute_block) actually supposed to do?
// * Which block authoring will be easiest to start with? Seems not babe because of the need to collect randomness in the runtime
// * Where does this core logic belong?
    // let next_state = match (previous_state, transaction) {
    //     (_, Set) => true,
    //     (_ Clear) => false,
    //     (prev_state, Toggle) => !prev_state
    // };

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
// Shouldn't be necessary if we're not using construct_runtime
// #![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

// use babe::SameAuthoritiesForever;
use primitives::OpaqueMetadata;
use rstd::prelude::*;
use sp_api::impl_runtime_apis;
use sp_runtime::traits::{
    BlakeTwo256, Block as BlockT, /*ConvertInto, NumberFor, StaticLookup,*/ Verify,
};
use sp_runtime::{
    ApplyExtrinsicResult,
    transaction_validity::{TransactionValidity, TransactionLongevity, ValidTransaction},
    generic,
    create_runtime_str,
	// impl_opaque_keys,
    AnySignature
};
#[cfg(feature = "std")]
use version::NativeVersion;
use version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
pub use balances::Call as BalancesCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use support::{traits::Randomness, StorageValue};

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = AnySignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <Signature as Verify>::Signer;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
//pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = primitives::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datastructures.
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;

    // pub type SessionHandlers = Babe;

    // impl_opaque_keys! {
    //     pub struct SessionKeys {
    //         pub babe: Babe,
    //     }
    // }
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("frameless-runtime"),
    impl_name: create_runtime_str!("frameless-runtime"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
};

/// Constants for Babe.

/// Since BABE is probabilistic this is the average expected block time that
/// we are targetting. Blocks will be produced at a minimum duration defined
/// by `SLOT_DURATION`, but some slots will not be allocated to any
/// authority and hence no block will be produced. We expect to have this
/// block time on average following the defined slot duration and the value
/// of `c` configured for BABE (where `1 - c` represents the probability of
/// a slot being empty).
/// This value is only used indirectly to define the unit constants below
/// that are expressed in blocks. The rest of the code should use
/// `SLOT_DURATION` instead (like the timestamp pallet for calculating the
/// minimum period).
/// <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
pub const MILLISECS_PER_BLOCK: u64 = 6000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

pub const EPOCH_DURATION_IN_BLOCKS: u32 = 100;

// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// The main struct in this module. In frame this comes from `construct_runtime!`
pub struct Runtime;

/// The address format for describing accounts.
// pub type Address = <Indices as StaticLookup>::Source;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, FramelessTransaction>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

// I guess we won't need any of this when using our own unchecked extrinsic type
// The SignedExtension to the basic transaction logic.
// pub type SignedExtra = (
//     system::CheckVersion<Runtime>,
//     system::CheckGenesis<Runtime>,
// );
/// Unchecked extrinsic type as expected by this runtime.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum FramelessTransaction {
    Set,
    Clear,
    Toggle,
}
// TODO Have to implement Extrinsic, WrapperTypeEncode, WrapperTypeDecode
// pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;

/// Extrinsic type that has already been checked.
// pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various pallets.
// pub type Executive = executive::Executive<Runtime, Block, system::ChainContext<Runtime>, Runtime, AllModules>;

impl_runtime_apis! {
    // https://substrate.dev/rustdocs/master/sp_api/trait.Core.html
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            //TODO prolly need to do something with pre-runtime digest? This may be another reason to use
            // consensus that is totally out of the runtime.
            unimplemented!()
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            // Do I actually need to do anything here?
            // I'm struggling to find descriptions of what these APIs are actually supposed to _do_
            unimplemented!()
        }
    }

    // https://substrate.dev/rustdocs/master/sp_api/trait.Metadata.html
    // "The Metadata api trait that returns metadata for the runtime."
    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            // Runtime::metadata().into()
            // Maybe this one can be omitted or just return () or something?
            // Would be really cool to return something that makes polkadot-js api happy,
            // but that seems unlikely.
            unimplemented!()
        }
    }

    // https://substrate.dev/rustdocs/master/sc_block_builder/trait.BlockBuilderApi.html
    impl block_builder_api::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            // Executive::apply_extrinsic(extrinsic)
            // Maybe this is where the core flipping logic goes?
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            // Docs say "Finish the current block." I guess that means I'm supposed to calculate
            // the new state root and stuff
            // Comment in executive says "It is up the caller to ensure that all header fields are valid except state-root."
            // Docs for return type
            // https://substrate.dev/rustdocs/master/sp_runtime/generic/struct.Header.html
            // Header {
            //     parent_hash: Hash::Output,
            //     number: Number,
            //     state_root: Hash::Output,
            //     extrinsics_root: Hash::Output,
            //     digest: Digest<Hash::Output>,
            // }
            unimplemented!()
        }

        fn inherent_extrinsics(data: inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            // I'm not using any inherents, so I guess I'll just return an empty vec
            Vec::new()
        }

        fn check_inherents(
            block: Block,
            data: inherents::InherentData
        ) -> inherents::CheckInherentsResult {
            // I'm not using any inherents, so it should be safe to just return ok
            inherents::CheckInherentsResult::new()
        }

        fn random_seed() -> <Block as BlockT>::Hash {
            // Lol how bad is this? What actually depends on it?
            0.into()
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
            // Any transaction of the correct type is valid
            Ok(ValidTransaction{
                priority: 1.into(),
                requires: Vec::new(),
                provides: Vec::new(),
                longevity: TransactionLongevity::max_value(),
                propagate: true,
            })
        }
    }

    // Hopefully don't need this, not planning to support offchain workers
    // impl offchain_primitives::OffchainWorkerApi<Block> for Runtime {
    //     fn offchain_worker(number: NumberFor<Block>) {
    //         Executive::offchain_worker(number)
    //     }
    // }

    // Probably easier to just go with aura or PoW or manual seal
    // Maybe this can remain largely unchanged..
    // Will have to get the behavior from the babe pallet somehow.
    impl babe_primitives::BabeApi<Block> for Runtime {
        fn configuration() -> babe_primitives::BabeConfiguration {
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            babe_primitives::BabeConfiguration {
                slot_duration: Babe::slot_duration(),//could be hardcoded here
                epoch_length: EpochDuration::get(),//already hardcoded above, could be moved here for simplicity
                c: PRIMARY_PROBABILITY, // already hardcoded above
                genesis_authorities: Babe::authorities(),//could be hardcoded here
                randomness: Babe::randomness(), // This one might be tricky. Prolly needs to come from actual babe pallet
                secondary_slots: true,
            }
        }
    }

    // Hopefully don't need this one; not using sessions pallet...
    // impl substrate_session::SessionKeys<Block> for Runtime {
    //     fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
    //         opaque::SessionKeys::generate(seed)
    //     }
    // }
}
