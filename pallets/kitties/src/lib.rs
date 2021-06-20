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
        pub fn create(origin){                   //create 一个kitty
        let sender = ensure_signed(origin)?;     //对于所有可调用方法，首先就要判断它的签名
        let kitty_id = Self::next_kitty_id()?;   //找到新创建的kitty的id，这里定义一个方法来实现，因为找到下个id其它地方也可能会使用。比如breed时。
        let dna = Self::random_value(&sender);   //为kitty创建数据，即之前说的u8数组。创建方法需要些随机化的方式
        let kitty = Kitty(dna);                  //有了dna数据后需要存储到链上，此为存储的操作。首先根据dna创建这个数据结构。因为链上定义的type是Kitty（见29行），所以这里用Kitty实例化一个object。
        Self::insert_kitty(&sender, kitty_id, kitty);  //实例化kitty后insert到map里去。所以这里定义了一个方法来实现。好处也是可重用，比如breed也有相同的操作。
    }
    }
}

impl<T: Trait> Module<T> {
    fn insert_kitty(owner: &T::AccountId, kitty_id: KittyIndex, kitty: Kitty) {
        Kitties::insert(kitty_id, kitty); //首先向Kitties这个map里面增加key，value数值对
        KittiesCount::put(kitty_id + 1);  //因为新增加了一个kitty，count加1。
        <KittyOwners<T>>::insert(kitty_id, owner); //最后把owner保存到链上
    }

    fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError>{    //根据现在已有的kitty数量来找到id。
		let kitty_id = Self::kitties_count();
		if kitty_id == T::KittyIndex::max_value() {  //因为index是用的u32类型，所以可能会存在越界问题。所以判断如果达到最大值则返回一个错误。并把错误定义到decl_error中。
			return Err(Error::<T>::KittiesCountOverflow.into());
		}
		Ok(kitty_id)
	}

    fn random_value(sender : &T::AccountId) -> [u8; 16] {  //random value会根据一组数据。seed,account,index作为一个payload。用blake2_128这个哈希函数对内容进行哈希，最终结果是128个bit的一段数据，可以存放到u8，长度是16的数组里面去。
		let payload = (
			T::Randomness::random_seed(),	
			&sender,
			<frame_system::Module<T>>::extrinsic_index(),
		);
		payload.using_encoded(blake2_128)
	}

}


//测试用例
#[cfg(test)]
mod test {
    use super::*;
}