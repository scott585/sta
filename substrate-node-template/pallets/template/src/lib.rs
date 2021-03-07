#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, ensure,};
use frame_system::{self as system, ensure_signed};
use sp_std::prelude::*;
use frame_support::traits::Get;


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type MaxClaimLength: Get<u32>;
}






// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as TemplateModule {
		/// 证明的存储项目
        /// 它将证明映射到提出声明的用户以及声明的时间。
		//Proofs: map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);
		Proofs get(fn proofs): map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);

	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// Event emitted when a proof has been claimed. [who, claim]
		/// 当一个新的证明被添加到区块链中
        ClaimCreated(AccountId, Vec<u8>),
        /// Event emitted when a claim is revoked by the owner. [who, claim]
		/// 当证明被移除时
		ClaimRevoked(AccountId, Vec<u8>),
		///存在转让
		ClaimTransfered(AccountId, Vec<u8>, AccountId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// 该证明已经被声明
        ProofAlreadyClaimed,
        /// 该证明不存在，因此它不能被撤销
        NoSuchProof,
        /// 该证明已经被另一个账号声明，因此它不能被撤销
		NotProofOwner,
		
		ProofTooLong,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		  // 错误必须被初始化，如果它们被 Pallet 所使用。
		  type Error = Error<T>;

		  // 事件必须被初始化，如果它们被模块所使用。
		  fn deposit_event() = default;
  
		  /// 允许用户队未声明的证明拥有所有权
		  #[weight = 10_000]
		  fn create_claim(origin, proof: Vec<u8>) {
			  // 检查 extrinsic 是否签名并获得签名者
			  // 如果 extrinsic 未签名，此函数将返回一个错误。
			
			  let sender = ensure_signed(origin)?;
  
			  // 校验指定的证明是否被声明
			  ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);
  
			  // 从 FRAME 系统模块中获取区块号.
			  let current_block = <frame_system::Module<T>>::block_number();
  
			  // 存储证明：发送人与区块号
			  Proofs::<T>::insert(&proof, (&sender, current_block));
  
			  // 声明创建后，发送事件
			  Self::deposit_event(RawEvent::ClaimCreated(sender, proof));
		  }
  
		  /// 允许证明所有者撤回声明
		  #[weight = 10_000]
		  fn revoke_claim(origin, proof: Vec<u8>) {
			  //  检查 extrinsic 是否签名并获得签名者
			  // 如果 extrinsic 未签名，此函数将返回一个错误。
			  // https://substrate.dev/docs/en/knowledgebase/runtime/origin
			  let sender = ensure_signed(origin)?;
  
			  // 校验指定的证明是否被声明
			  ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);
  
			  // 获取声明的所有者
			  let (owner, _) = Proofs::<T>::get(&proof);
  
			  // 验证当前的调用者是证声明的所有者
			  ensure!(sender == owner, Error::<T>::NotProofOwner);
  
			  // 从存储中移除声明
			  Proofs::<T>::remove(&proof);
  
			  // 声明抹掉后，发送事件
			  Self::deposit_event(RawEvent::ClaimRevoked(sender, proof));
		  }

		  // 存证转让
		#[weight = 10_000]
		pub fn transfer_claim(origin, claim: Vec<u8>, receiver: T::AccountId) -> dispatch::DispatchResult {
			// 验签+获得调用者
			let sender = ensure_signed(origin)?;
			// 检测存证是否存在，如果不存在，报错
			ensure!(Proofs::<T>::contains_key(&claim), Error::<T>::NoSuchProof);
			// 获取存证信息，主要是需要获得存证拥有者
			let (owner, _block_number) = Proofs::<T>::get(&claim);
			// 检测拥有者和调用者是不是一个人，如果不是报错
			ensure!( owner == sender , Error::<T>::NotProofOwner);
			// 更新存证信息，将拥有人修改为指定的账号，用 insert ，因为是 hashmap 会自动实现为替换
			Proofs::<T>::insert(&claim,(receiver.clone(), system::Module::<T>::block_number()));
			// 触发存证转让的事件
			Self::deposit_event(RawEvent::ClaimTransfered(sender, claim, receiver));
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	pub fn do_create_claim(sender: T::AccountId, claim: Vec<u8>) -> Result<(), dispatch::DispatchError> {
		// 检测存证是否已经存在
		ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyClaimed);
		// 检测存证的长度是否超过限制
		ensure!( T::MaxClaimLength::get() >= claim.len() as u32, Error::<T>::ProofTooLong);
		// 存入存证数据
		Proofs::<T>::insert(&claim,(sender.clone(), system::Module::<T>::block_number()));
		// 触发存证写入成功的时间
		Self::deposit_event(RawEvent::ClaimCreated(sender, claim));
		Ok(())
	}
}