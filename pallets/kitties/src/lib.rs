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


// 以下是pallet常规的开发
pub trait Trait: frame_system::Trait {

}


decl_storage! {
    trait Store for Module<T: Trait> as kitties {

    }
}

decl_error!{
    pub enum Error for Module<T: Trait> {

    }
}

decl_event!(
    pub enum Event<T> where <T as frame_system::Trait>::AccountId, {

    }
)

impl<T: Trait> Module<T> {

}


//测试用例
#[cfg(test)]
mod test {
    use super::*;
}