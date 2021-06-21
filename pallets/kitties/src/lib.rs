#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::Randomness,
    traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
    Parameter, StorageMap, StorageValue,
};
use frame_system::ensure_signed;
use sp_io::hashing::blake2_128;
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

decl_error! {
    pub enum Error for Module<T: Trait> {
        KittiesCountOverflow,
        InvalidKittyId,
        RequireDifferentParent,
    }
}

decl_event!(
    pub enum Event<T> where <T as frame_system::Trait>::AccountId, {
        Created(AccountId, KittyIndex),
        Transferred(AccountId, AccountId, KittyIndex),    // 定义transfer的event
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;
        // (最先放错位置在impl module里了，现在改为放在decl_module里)首次build时报错，event的定义没放到module里去。可能忽略了两个东西，一个是deposit event这个实现。二是之前metadata里说过，error最好是在decal module里进行绑定，这样metadata会包含这些信息。所以加入以上这两行代码
        #[weight=0]
        pub fn create(origin){                   //create 一个kitty
        let sender = ensure_signed(origin)?;     //对于所有可调用方法，首先就要判断它的签名
        let kitty_id = Self::next_kitty_id()?;   //找到新创建的kitty的id，这里定义一个方法来实现，因为找到下个id其它地方也可能会使用。比如breed时。
        let dna = Self::random_value(&sender);   //为kitty创建数据，即之前说的u8数组。创建方法需要些随机化的方式
        let kitty = Kitty(dna);                  //有了dna数据后需要存储到链上，此为存储的操作。首先根据dna创建这个数据结构。因为链上定义的type是Kitty（见29行），所以这里用Kitty实例化一个object。
        Self::insert_kitty(&sender, kitty_id, kitty);  //实例化kitty后insert到map里去。所以这里定义了一个方法来实现。好处也是可重用，比如breed也有相同的操作。
        Self::deposit_event(RawEvent::Created(sender, kitty_id)); //对于create，希望有一些event可以抛出来，这样ui或其它一些应用可以知道kitty已经被创建好这个消息，所以最后会把这个event存储起来。event的定义叫created(不懂)。
    }

    #[weight = 0]
    pub fn transfer(origin, to: T::AccountId, Kitty_id: KittyIndex) { //定义transfer，这里指定了to的owner是谁，以及对哪个kitty进行操作
        let sender = ensure_signed(origin)?; //判断方法签名
        <KittyOwners<T>>::insert(kitty_id, to.clone());  // 把kitty的新owner放到map里去
        Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));    // 操作成功后抛出event
    }

    #[weight = 0]
    pub fn breed(origin, kitty_id_1: KittyIndex, kitty_id_2: KittyIndex) {       // 定义breed。由于这个实现比较复杂，所以我们倾向于把这个实现在impl module里做，而不是decl module里。
        let sender = ensure_signed(origin)?; //先判断必须有个合法的签名
        let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;    //定义一个do breed方法，由它来调用。
        Self::deposit_event(RawEvent::Created(sender, new_kitty_id)); //这里的event为了简化就借用了create的。所以无法查看parent，如果有需求的话可以创建一个新的event。

    }


    }
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
    // 这个function只是个简单的算法实现，并不需要知道module的trait type之类的，也不需要它的数据。所以可以放到impl module外面
    (selector & dna1) | (!selector & dna2)
}

impl<T: Trait> Module<T> {
    fn insert_kitty(owner: &T::AccountId, kitty_id: KittyIndex, kitty: Kitty) {
        Kitties::insert(kitty_id, kitty); //首先向Kitties这个map里面增加key，value数值对
        KittiesCount::put(kitty_id + 1); //因为新增加了一个kitty，count加1。
        <KittyOwners<T>>::insert(kitty_id, owner); //最后把owner保存到链上
    }

    fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
        //根据现在已有的kitty数量来找到id。
        let kitty_id = Self::kitties_count();
        if kitty_id == T::KittyIndex::max_value() {
            //因为index是用的u32类型，所以可能会存在越界问题。所以判断如果达到最大值则返回一个错误。并把错误定义到decl_error中。
            return Err(Error::<T>::KittiesCountOverflow.into());
        }
        Ok(kitty_id)
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        //random value会根据一组数据。seed,account,index作为一个payload。用blake2_128这个哈希函数对内容进行哈希，最终结果是128个bit的一段数据，可以存放到u8，长度是16的数组里面去。
        let payload = (
            T::Randomness::random_seed(),
            &sender,
            <frame_system::Module<T>>::extrinsic_index(),
        );
        payload.using_encoded(blake2_128)
    }

    fn do_breed(
        sender: &T::AccountId,
        kitty_id_1: KittyIndex,
        kitty_id2: T::KittyIndex,
    ) -> sp_std::result::Result<KittyIndex, DispatchError> {
        //这段代码最后部分Result后视频显示不全，所以使用的是SubstrateStarter库中的片段
        let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
        //查看parent id是否存在，如果不存在就抛出错误，交易失败。错误定义为InvalidKittyId
        let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;
        //查看parent id是否存在，如果不存在就抛出错误，交易失败。错误定义为InvalidKittyId
        ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);
        // 判断两个id是否相同
        let kitty_id = Self::next_kitty_id()?;
        // 定义完之前的错误，则可获得一个新的id
        let kitty1_dna = kitty1.0;
        let kitty2_dna = kitty2.0;
        // 获得两个dna的数据
        let selector = Self::random_value(&sender);
        // 用随机数，如果随机数某个bit是0，则用kitty1的dna，如果是1，则用kitty2的相应那一位
        let mut new_dna = [0u8; 16];
        // 定义数据结构去存放新的dna
        for i in 0..kitty1_dna.len() {
            new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
        }
        Self::insert_kitty(sender, kitty_id, Kitty(new_dna));
        Ok(kitty_id)
    }
}

//测试用例
#[cfg(test)]
mod test {
    use super::*;

    use frame_support::{
        impl_outer_origin, parameter_types,
        traits::{OnFinalize, OnInitialize},
        weights::Weight,
    };
    use frame_system as system;
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        Perbill,
    };

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }

    impl system::Trait for Test {
        type BaseCallFilter = ();
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type DbWeight = ();
        type BlockExecutionWeight = ();
        type ExtrinsicBaseWeight = ();
        type MaximumExtrinsicWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type PalletInfo = ();
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
    }

    type Randomness = pallet_randomness_collective_flip::Module<Test>;

    impl Trait for Test {
        type Event = ();
        type Randomness = Randomness;
    }

    pub type Kitties = Module<Test>;
}
