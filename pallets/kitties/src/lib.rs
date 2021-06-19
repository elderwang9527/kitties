#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{decl_module,decl_storage, decl_event, decl_error, StorageValue, ensure, StorageMap, traits::Randomness, Parameter,traits::{ExistenceRequirement ,Get, Currency, ReservableCurrency}
};
use sp_io::hashing::blake2_128;
use frame_system::ensure_signed;
use sp_runtime::DispatchError;

type KittyIndex = u32;
#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);

