/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use parity_codec::Encode;
use support::{decl_storage, decl_module, StorageValue, StorageMap,
    dispatch::Result, ensure, decl_event};
use system::ensure_signed;
use runtime_primitives::traits::{Hash};
use parity_codec_derive;//::{Encode, Decode};

use rstd::prelude::*;

/// The module's configuration trait.
pub trait Trait: system::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(parity_codec_derive::Encode, parity_codec_derive::Decode, Default, Clone, PartialEq)]
pub struct Asset<Hash> {
	id: Hash,
	name: Vec<u8>,
	open: bool,
}

/// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as bdevux {

			// ----------- オリジナル資産管理

			/// オリジナル資産
			Assets get(asset): map T::Hash => Asset<T::Hash>;
			/// オリジナル資産オーナ
			// オリジナル資産のオーナは 1 人
			AssetOwner get(owner_of): map T::Hash => Option<T::AccountId>;

			/// 全オリジナル資産配列 インデックス => 資産ID
			AllAssetsArray get(asset_by_index): map u64 => T::Hash;
			/// 全オリジナル資産数
			AllAssetsCount get(all_asset_count): u64;
			/// 全オリジナル資産配列におけるインデックス  資産ID => オリジナル資産インデックス
			AllAssetsIndex: map T::Hash => u64;

			/// 発行したオリジナル資産配列
			OwnedAssetsArray get(asset_of_owner_by_index): map (T::AccountId, u64) => T::Hash;
			/// 発行したオリジナル資産数
			OwnedAssetsCount get(owned_asset_count): map T::AccountId => u64;
			/// 発行したオリジナル資産インデックス
			// オーナーは 1 人であるため AccountId は不要
			OwnedAssetsIndex: map T::Hash => u64;

			/// 発行済オリジナル資産量
			/// 資産 ID => 発行量
			/// 資産を追加発行した場合は、この値を更新する
			TotalIssuedAssets get(total_issued_asset): map T::Hash => u64;

			Nonce: u64;

			// ----------- オリジナル資産管理 --- ここまで

			// ----------- 所有している資産の管理
			
			/// インデックス => 資産 ID
			MyAssetsArray get(my_asset_by_index): map (T::AccountId, u64) => T::Hash;
			/// 所有している資産数
			MyAssetsCount get(my_asset_count): map T::AccountId => u64;
			/// 所有している資産の配列におけるインデックス
			MyAssetsIndex: map (T::AccountId, T::Hash) => u64;
			/// 所有資産量
			/// 資産 ID => 所有量
			MyAssetBalances get(my_asset_balance): map (T::AccountId, T::Hash) => u64;

			// ----------- 所有している資産の管理 --- ここまで

	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event<T>() = default;

		/// オリジナル資産発行（作成）
		/// 関数名は MultiChain に合わせている
		/// name: 資産名
		/// issue_qty: 初期発行量, 発行先は関数呼び出しアドレス
		/// open: true であれば追加発行可能
		fn issue(origin, name: Vec<u8>, issue_qty: u64, open: bool) -> Result {

				// 関数呼び出し者
				let sender = ensure_signed(origin)?;

				// 発行済資産数
				let owned_asset_count = Self::owned_asset_count(&sender);
				// 発行済資産数 + 1
				let new_owned_asset_count = owned_asset_count.checked_add(1)
						.ok_or("Overflow adding a new Asset to account balance")?;

				// 全資産数
				let all_asset_count = Self::all_asset_count();
				// 全資産数 + 1
				let new_all_asset_count = all_asset_count.checked_add(1)
						.ok_or("Overflow adding a new Asset to total supply")?;

				// 資産 ID 生成
				let nonce = <Nonce<T>>::get();
				let random_hash = (<system::Module<T>>::random_seed(), &sender, nonce)
						.using_encoded(<T as system::Trait>::Hashing::hash);

				ensure!(!<AssetOwner<T>>::exists(random_hash), "Asset already exists");

				// オリジナル資産情報
				let new_asset = Asset {
						id: random_hash,
						name: name,
						open: open
				};

				// 資産発行
				let my_asset_count = Self::my_asset_count(&sender);
				let new_my_asset_count = my_asset_count.checked_add(1)
						.ok_or("Overflow adding a new My Asset to total supply")?;


				// --------------------- 更新
				// 更新する値が正常であることが確認済みであることが必須!

				<Assets<T>>::insert(random_hash, new_asset);
				<AssetOwner<T>>::insert(random_hash, &sender);

				<AllAssetsArray<T>>::insert(all_asset_count, random_hash);
				<AllAssetsCount<T>>::put(new_all_asset_count);
				<AllAssetsIndex<T>>::insert(random_hash, all_asset_count);

				<OwnedAssetsArray<T>>::insert((sender.clone(), owned_asset_count), random_hash);
				<OwnedAssetsCount<T>>::insert(&sender, new_owned_asset_count);
				<OwnedAssetsIndex<T>>::insert(random_hash, owned_asset_count);

				<TotalIssuedAssets<T>>::insert(random_hash, issue_qty);

				<Nonce<T>>::mutate(|n| *n += 1);

				<MyAssetsArray<T>>::insert((sender.clone(), my_asset_count), random_hash);
				<MyAssetsCount<T>>::insert(&sender, new_my_asset_count);
				<MyAssetsIndex<T>>::insert((sender.clone(), random_hash), my_asset_count);
				<MyAssetBalances<T>>::insert((sender.clone(), random_hash), issue_qty);

				// --------------------- 更新 --- ここまで
				
				Self::deposit_event(RawEvent::Issued(sender, random_hash));

				Ok(())
		}

		/// オリジナル資産追加発行
		/// 関数名は MultiChain に合わせている
		/// asset_id: 資産 ID
		/// issue_qty: 発行量
		/// asset.open が false の場合は失敗する
		fn issuemore(origin, asset_id: T::Hash, issue_qty: u64) -> Result {
				let sender = ensure_signed(origin)?;

				// 存在確認
				ensure!(<Assets<T>>::exists(asset_id), "This asset does not exist");
				
				// 所有者（発行者）確認
				let owner = Self::owner_of(asset_id).ok_or("No owner for this asset")?;
				ensure!(owner == sender, "You do not own this asset");

				// 追加発行確認
				let asset = Self::asset(asset_id);
				ensure!(asset.open == true, "You can not issue more");

				// 全発行量
				let total_issued_asset = Self::total_issued_asset(asset_id);
				let new_total_issued_asset = total_issued_asset.checked_add(issue_qty)
						.ok_or("Overflow adding a new Asset")?;
				
				// 追加発行した資産を現在の資産に加算
				let my_asset_balance = Self::my_asset_balance((sender.clone(), asset_id));
				let new_my_asset_balance = my_asset_balance.checked_add(issue_qty)
						.ok_or("Overflow adding a new Asset")?;

				// --------------------- 更新
				// 更新する値が正常であることが確認済みであることが必須!
				<TotalIssuedAssets<T>>::insert(asset_id, new_total_issued_asset);
				<MyAssetBalances<T>>::insert((sender.clone(), asset_id), new_my_asset_balance);

				// --------------------- 更新 --- ここまで

				// イベント
				Self::deposit_event(RawEvent::IssuedMore(sender, asset_id, issue_qty));

				Ok(())
		}

		/// 資産送信
		///
		/// # Arguments
		///
		/// `to` - 送信先アドレス
		/// `asset_id` - 資産 ID
		/// `qty` - 送信量
		/// 
		/// # Arguments
		fn sendasset(origin, to: T::AccountId, asset_id: T::Hash, qty: u64) -> Result {
				// 署名確認
				let sender = ensure_signed(origin)?;

				// 所有確認
				// - 資産確認
				ensure!(<MyAssetsIndex<T>>::exists((sender.clone(), asset_id)), "This asset does not exist");
				// - 送信額確認
				let my_asset_balance = Self::my_asset_balance((sender.clone(), asset_id));
				ensure!(my_asset_balance >= qty, "Your asset is less than you want to send the amount.");
				
				// -- 受信者資産
				let flg = <MyAssetsIndex<T>>::exists((to.clone(), asset_id));
				let to_asset_balance = if flg {
						Self::my_asset_balance((to.clone(), asset_id))
				} else {
						0
				};

				// 送信者資産
				let new_my_asset_balance = my_asset_balance.checked_sub(qty)
						.ok_or("Your asset is less than you want to send the amount.")?;
				// 受信者資産
				let new_to_asset_balance = to_asset_balance.checked_add(qty)
						.ok_or("Overflow adding (to)'s asset")?;


				// if 相手が資産を持っていない場合は資産情報を追加
				if !flg {
						let to_asset_count = Self::my_asset_count(&to);
						let new_to_asset_count = to_asset_count.checked_add(1)
								.ok_or("Overflow adding a new My Asset to total supply")?;

				// --------------------- 更新
				// 更新する値が正常であることが確認済みであることが必須!

						<MyAssetBalances<T>>::insert((sender.clone(), asset_id), new_my_asset_balance);


						<MyAssetsArray<T>>::insert((to.clone(), to_asset_count), asset_id);
						<MyAssetsCount<T>>::insert(&to, new_to_asset_count);
						<MyAssetsIndex<T>>::insert((to.clone(), asset_id), to_asset_count);
						<MyAssetBalances<T>>::insert((to.clone(), asset_id), new_to_asset_balance);
				// --------------------- 更新 --- ここまで
				} else {
				// --------------------- 更新
				// 更新する値が正常であることが確認済みであることが必須!
						<MyAssetBalances<T>>::insert((sender.clone(), asset_id), new_my_asset_balance);
						<MyAssetBalances<T>>::insert((to.clone(), asset_id), new_to_asset_balance);
				// --------------------- 更新 --- ここまで
				}            

				// イベント
				Self::deposit_event(RawEvent::SentAsset(sender, to, asset_id, qty));

				Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where 
		AccountId = <T as system::Trait>::AccountId ,
		Hash = <T as system::Trait>::Hash
		{
		/// オリジナル資産発行
		Issued(AccountId, Hash),
		/// 追加発行
		IssuedMore(AccountId, Hash, u64),
		/// 資産送信
		SentAsset(AccountId, AccountId, Hash, u64),
	}
);

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok, assert_err};
	use runtime_primitives::{
		BuildStorage,
		traits::{BlakeTwo256, IdentityLookup},
		testing::{Digest, DigestItem, Header}
	};

	const ACCOUNT_1: u64 = 1;
	const ACCOUNT_2: u64 = 2;

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	impl system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Digest = Digest;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type Log = DigestItem;
	}
	impl Trait for Test {
		type Event = ();
	}
	type bdevux = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			// assert_ok!(bdevux::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			// assert_eq!(bdevux::something(), Some(42));
		});
	}
	#[test]
	fn issue_should_work() {
		with_externalities(&mut new_test_ext(), || {
			assert_ok!(bdevux::issue(Origin::signed(ACCOUNT_1), b"bdevux".to_vec(), 1001, true));
			assert_eq!(bdevux::all_asset_count(), 1);
			assert_eq!(<Nonce<Test>>::get(), 1);
			assert_ok!(bdevux::issue(Origin::signed(ACCOUNT_2), b"bdevux2".to_vec(), 1002, false));
			assert_eq!(bdevux::all_asset_count(), 2);
			assert_eq!(<Nonce<Test>>::get(), 2);
			assert_ok!(bdevux::issue(Origin::signed(ACCOUNT_2), b"bdevux3".to_vec(), 1003, false));
			assert_eq!(bdevux::all_asset_count(), 3);
			assert_eq!(<Nonce<Test>>::get(), 3);

			assert_eq!(bdevux::owned_asset_count(ACCOUNT_1), 1);
			assert_eq!(bdevux::owned_asset_count(ACCOUNT_2), 2);


			let hash = bdevux::asset_by_index(0);
			let asset = bdevux::asset(hash);
			assert_eq!(asset.id, hash);
			assert_eq!(asset.name, b"bdevux".to_vec());
			assert_eq!(asset.open, true);
			assert_eq!(bdevux::owner_of(hash), Some(ACCOUNT_1));
			assert_eq!(bdevux::asset_of_owner_by_index((ACCOUNT_1, 0)), hash);
			assert_eq!(bdevux::total_issued_asset(hash), 1001);
			assert_eq!(bdevux::my_asset_by_index((ACCOUNT_1, 0)), hash);
			assert_eq!(bdevux::my_asset_count(ACCOUNT_1), 1);
			assert_eq!(bdevux::my_asset_balance((ACCOUNT_1, hash)), 1001);
			assert_eq!(<AllAssetsIndex<Test>>::get(hash), 0);
			assert_eq!(<OwnedAssetsIndex<Test>>::get(hash), 0);
			assert_eq!(<MyAssetsIndex<Test>>::get((ACCOUNT_1, hash)), 0);

			let hash = bdevux::asset_by_index(1);
			let asset = bdevux::asset(hash);
			assert_eq!(asset.id, hash);
			assert_eq!(asset.name, b"bdevux2".to_vec());
			assert_eq!(asset.open, false);
			assert_eq!(bdevux::owner_of(hash), Some(ACCOUNT_2));
			assert_eq!(bdevux::asset_of_owner_by_index((ACCOUNT_2, 0)), hash);
			assert_eq!(bdevux::total_issued_asset(hash), 1002);
			assert_eq!(bdevux::my_asset_by_index((ACCOUNT_2, 0)), hash);
			assert_eq!(bdevux::my_asset_count(ACCOUNT_2), 2);
			assert_eq!(bdevux::my_asset_balance((ACCOUNT_2, hash)), 1002);
			assert_eq!(<AllAssetsIndex<Test>>::get(hash), 1);
			assert_eq!(<OwnedAssetsIndex<Test>>::get(hash), 0);
			assert_eq!(<MyAssetsIndex<Test>>::get((ACCOUNT_2, hash)), 0);

			let hash = bdevux::asset_by_index(ACCOUNT_2);
			let asset = bdevux::asset(hash);
			assert_eq!(asset.id, hash);
			assert_eq!(asset.name, b"bdevux3".to_vec());
			assert_eq!(asset.open, false);
			assert_eq!(bdevux::owner_of(hash), Some(ACCOUNT_2));
			assert_eq!(bdevux::asset_of_owner_by_index((ACCOUNT_2, 1)), hash);
			assert_eq!(bdevux::total_issued_asset(hash), 1003);
			assert_eq!(bdevux::my_asset_by_index((ACCOUNT_2, 1)), hash);
			assert_eq!(bdevux::my_asset_count(ACCOUNT_2), 2);
			assert_eq!(bdevux::my_asset_balance((ACCOUNT_2, hash)), 1003);
			assert_eq!(<AllAssetsIndex<Test>>::get(hash), 2);
			assert_eq!(<OwnedAssetsIndex<Test>>::get(hash), 1);
			assert_eq!(<MyAssetsIndex<Test>>::get((ACCOUNT_2, hash)), 1);


		});
	}

	#[test]
	fn issuemore_should_work_open() {
		with_externalities(&mut new_test_ext(), || {

			let first: u64 = 1<<60;
			let issue_qty = 5000;
			
			assert_ok!(bdevux::issue(Origin::signed(ACCOUNT_1), b"bdevux".to_vec(), first, true));

			assert_err!(bdevux::issuemore(Origin::signed(ACCOUNT_1), H256::default(), issue_qty), "This asset does not exist");

			let hash = bdevux::asset_by_index(0);
			assert_ok!(bdevux::issuemore(Origin::signed(ACCOUNT_1), hash, issue_qty));
			assert_err!(bdevux::issuemore(Origin::signed(ACCOUNT_2), hash, issue_qty), "You do not own this asset");

			assert_eq!(bdevux::my_asset_balance((ACCOUNT_1, hash)), first + issue_qty);
			assert_eq!(bdevux::total_issued_asset(hash), first + issue_qty);

		});
	}
	#[test]
	fn issuemore_should_work_not_open() {
		with_externalities(&mut new_test_ext(), || {

			let first: u64 = 1<<60;
			let issue_qty = 5000;
			
			assert_ok!(bdevux::issue(Origin::signed(ACCOUNT_1), b"bdevux".to_vec(), first, false));

			assert_err!(bdevux::issuemore(Origin::signed(ACCOUNT_1), H256::default(), issue_qty), "This asset does not exist");

			let hash = bdevux::asset_by_index(0);
			assert_err!(bdevux::issuemore(Origin::signed(ACCOUNT_2), hash, issue_qty), "You do not own this asset");
			assert_err!(bdevux::issuemore(Origin::signed(ACCOUNT_1), hash, issue_qty), "You can not issue more");

			assert_eq!(bdevux::my_asset_balance((ACCOUNT_1, hash)), first);
			assert_eq!(bdevux::total_issued_asset(hash), first);

		});
	}
	#[test]
	fn issuemore_should_work_open_overflow() {
		with_externalities(&mut new_test_ext(), || {

			let first: u64 = std::u64::MAX - 100;
			let issue_qty = 101;

			assert_ok!(bdevux::issue(Origin::signed(ACCOUNT_1), b"bdevux".to_vec(), first, true));
			let hash = bdevux::asset_by_index(0);
			assert_err!(bdevux::issuemore(Origin::signed(ACCOUNT_1), hash, issue_qty), "Overflow adding a new Asset");

			assert_eq!(bdevux::my_asset_balance((ACCOUNT_1, hash)), first);
			assert_eq!(bdevux::total_issued_asset(hash), first);

		});
	}

		#[test]
	fn sendasset_should_work() {
		with_externalities(&mut new_test_ext(), || {

			let first: u64 = 1<<60;
			let qty = 1_000_000;

			assert_ok!(bdevux::issue(Origin::signed(ACCOUNT_1), b"bdevux".to_vec(), first, true));
			let hash = bdevux::asset_by_index(0);
			assert_eq!(bdevux::my_asset_balance((ACCOUNT_1, hash)), first);

			assert_ok!(bdevux::sendasset(Origin::signed(ACCOUNT_1), ACCOUNT_2, hash, qty));

			assert_eq!(bdevux::my_asset_balance((ACCOUNT_1, hash)), first - qty);
			assert_eq!(bdevux::my_asset_balance((ACCOUNT_2, hash)), qty);
			assert_eq!(bdevux::my_asset_by_index((ACCOUNT_2, 0)), hash);
			assert_eq!(bdevux::my_asset_count(ACCOUNT_2), 1);
			assert_eq!(<MyAssetsIndex<Test>>::get((ACCOUNT_2, hash)), 0);

		});
	}
	
	#[test]
	fn sendasset_should_work_less() {
		with_externalities(&mut new_test_ext(), || {

			let first: u64 = 1_000_000;
			let qty = 1_000_001;

			assert_ok!(bdevux::issue(Origin::signed(ACCOUNT_1), b"bdevux".to_vec(), first, true));
			let hash = bdevux::asset_by_index(0);
			assert_eq!(bdevux::my_asset_balance((ACCOUNT_1, hash)), first);

			assert_err!(bdevux::sendasset(Origin::signed(ACCOUNT_1), ACCOUNT_2, hash, qty), "Your asset is less than you want to send the amount.");

			assert_eq!(bdevux::my_asset_balance((ACCOUNT_1, hash)), first);
			assert_eq!(bdevux::my_asset_balance((ACCOUNT_2, hash)), 0);
			assert_eq!(bdevux::my_asset_by_index((ACCOUNT_2, 0)), H256::default());
			assert_eq!(bdevux::my_asset_count(ACCOUNT_2), 0);
			assert_eq!(<MyAssetsIndex<Test>>::get((ACCOUNT_2, hash)), 0);

		});
	}
}
