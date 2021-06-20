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

//所有的pallet都需要用到system包含的trait
pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    //产生kitty的dna数据时需要一些随机的函数
    type Randomness: Randomness<Self::Hash>;

}


decl_storage! {
    trait Store for Module<T: Trait> as kitties {
        //用于存储kitty数据，key是它的id
        pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty>;
        //记录到了第几个kitty
        pub KittiesCount get(fn kitties_count): T::KittyIndex;
        //kitty的owner
        pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;
    }
}

decl_error!{
    pub enum Error for Module<T: Trait> {
        KittiesCountOverflow,
    }
}

decl_event!(
    pub enum Event<T> where <T as frame_system::Trait>::AccountId, {

    }
)

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        #[weight=0]
        pub fn create(origin){  //create 一个kitty
        let sender = ensure_signed(origin)?;    //对于所有可调用方法，首先就要判断它的签名
        let kitty_id = Self::next_kitty_id()?;    //找到新创建的kitty的id，这里定义一个方法来实现，因为找到下个id其它地方也可能会使用。比如breed时。
    }
    }
}

impl<T: Trait> Module<T> {
    fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError>{    //根据现在已有的kitty数量来找到id。
		let kitty_id = Self::kitties_count();
		if kitty_id == T::KittyIndex::max_value() {  //因为index是用的u32类型，所以可能会存在越界问题。所以判断如果达到最大值则返回一个错误。并把错误定义到decl_error中。
			return Err(Error::<T>::KittiesCountOverflow.into());
		}
		Ok(kitty_id)
	}

}


//测试用例
#[cfg(test)]
mod test {
    use super::*;
}