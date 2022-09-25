#![cfg_attr(not(feature = "std"), no_std)]
// #![allow(unused_imports)]
// #![allow(unused_variables)]
// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

//   #[allow(unused_imports)]
  use frame_support::pallet_prelude::*;
  use frame_system::pallet_prelude::*;
  // use frame_support::inherent::Vec;
//   use scale_info::{prelude::string::String, Type};
	use frame_support::{
		inherent::Vec,
		traits::{tokens::ExistenceRequirement, Currency, Randomness},
	};
	use scale_info::TypeInfo;
	use sp_core::{
		// crypto::AccountId32, 
		sr25519};
	use frame_support::sp_runtime::SaturatedConversion;


  #[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	// type Balance = u32;



  // Struct for holding File information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct File<T: Config> {
		pub price: Option<BalanceOf<T>>,
		pub owner: AccountOf<T>,
		pub file_type : FileType,
		pub file_link: BoundedVec<u8, T::MaxLength>,
		pub allow_download :bool,
		pub file_size : u64,
	}

	 // Set FileType type in File struct.
	 #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	 #[scale_info(skip_type_params(T))]
	 #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	 pub enum FileType {
		 Normal,
		 Privileged,
	 }
    

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);

  /// Configure the pallet by specifying the parameters and types on which it depends.
  #[pallet::config]
  pub trait Config: frame_system::Config {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

	type AccountId1: From<sr25519::Public> + IsType<<Self as frame_system::Config>::AccountId>;
	
    /// The Currency handler for the FileStorage pallet.
	type Currency: Currency<Self::AccountId>;

	/// The maximum number of files a single account can own.
	#[pallet::constant]
	type MaxFileOwned: Get<u32>;

	/// The minimum length a file_link may be.
	#[pallet::constant]
	type MinLength: Get<u32>;
	/// The maximum length a file_link may be.
	#[pallet::constant]
	type MaxLength: Get<u32>;


	/// The type of Randomness we want to specify for this pallet.
	type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
  }
  
  
  // Pallets use events to inform users when important changes are made.
  // Event documentation should end with an array that provides descriptive names for parameters.
  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    ///Event emitted when a file is uploaded 
    FileCreated { who: T::AccountId, cid: T::Hash },
	///Event file Downloaded
	FileDownloaded {cid: T::Hash, count: u64},
	///Event ownership changed
	FileOwnerChanged {cid: T::Hash, new_owner: T::AccountId},
  }
  
  
  #[pallet::error]
  pub enum Error<T> {
	///already uploaded
	AlreadyUploaded,
	///file does not exist
	FileDoesNotExist,
	///link of the file is too long
	LinkTooLong,
	///link of the file is too short
	LinkTooShort,
	///sender is not the owner of the file
	SenderIsNotOwner,
	///file download not allowed at the time of upload
	FileNotDownloadable,
	///low balance
	NotEnoughBalance,
  }

  #[pallet::storage]
	#[pallet::getter(fn file_cnt)]
	/// Keeps track of the number of Files in existence.
	pub(super) type FileCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn files)]
	/// Stores a Files's unique traits, owner and price.
	pub(super) type Files<T: Config> = StorageMap<_, Twox64Concat, T::Hash, File<T>>;

	#[pallet::storage]
	#[pallet::getter(fn files_owned)]
	/// Keeps track of what accounts own what File.
	pub(super) type FilesOwned<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::Hash,
		T::AccountId,
		// ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn files_download_cnt)]
	/// Keeps track of count of downloads file wise.
	pub(super) type FilesDownloadCnt<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::Hash,
		u64,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn total_download_cnt)]
	/// Keeps track of total number of downloads.
	pub(super) type TotalDownloadCount<T: Config> = StorageValue<
		_,
		u64,
		ValueQuery,
	>;
	

	#[pallet::storage]
	#[pallet::getter(fn file_downloaders)]
	/// Keeps track of what accounts downloaded a file
	pub(super) type FileDownloaders<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::Hash,
		Blake2_128Concat,
		T::AccountId,
		u64,
		ValueQuery,
	>;


    #[pallet::call]
    impl<T: Config> Pallet<T> {


        /// Upload File and sets its properties and updates storage.
		#[pallet::weight(100)]
		pub fn create_file(
			origin: OriginFor<T>,
			cid: T::Hash,
			file_link: Vec<u8>,
			allow_download :bool,
			file_type: Option<FileType>,
			cost: Option<BalanceOf<T>>, //input considered only if file is privileged
            file_size: u64,
		) -> DispatchResult {
			let sender: T::AccountId = ensure_signed(origin)?;

            //Action: checking if the file already created
            ensure!(!Files::<T>::contains_key(&cid), Error::<T>::AlreadyUploaded);
      
	        let bounded_file_link: BoundedVec<_, _> =
	      			file_link.try_into().map_err(|()| Error::<T>::LinkTooLong)?;
	        ensure!(bounded_file_link.len() >= T::MinLength::get() as usize, Error::<T>::LinkTooShort);
      
            //keeping these const in local here unlike const MinLength intentionally
			const ALLOWD_SIZE_FREE:u64 = 250;
			const RATE_PER_UNIT:u64 = 100_000_000_000; // 0.1 DOT (say) 

			let cost_of_file = 
				match file_type.clone().unwrap_or_else(|| FileType::Normal) {
					FileType::Normal => {  // calculate the cost of normal file as per size and free limits
						//calculate the price in u64 
						let cost_in_u64:u64 =  if file_size <= ALLOWD_SIZE_FREE { 
							0 
						} else { 
								(file_size - ALLOWD_SIZE_FREE) * RATE_PER_UNIT 			
							};
							// convert cost into type Balance
							let new_cost: Option<BalanceOf<T>> = Some(cost_in_u64.saturated_into::<BalanceOf<T>>());
							new_cost
					},
					FileType::Privileged => cost, // keep the cost of the file as it is for privileged file
					
				};

			//create File data
			let file: File<T> = File::<T> {
				price: cost_of_file,
				file_type: file_type.unwrap_or_else(|| FileType::Normal),
				owner: sender.clone(),
				file_link: bounded_file_link,
				allow_download,
				file_size,
				};

			//insert file  into the storage
	        <Files<T>>::insert(&cid, file);

			//update number of total files uploaded
	        let mut cnt = <FileCnt<T>>::get();
	        cnt+=1;
	        <FileCnt<T>>::set(cnt);
      
	        //update owner of the files
	        <FilesOwned<T>>::insert(&cid, &sender);
      
            // Deposite file created event
            Self::deposit_event(Event::FileCreated { who: sender, cid });
	      	Ok(())
	    }




        /// Download File .
		#[pallet::weight(100)]
		pub fn download_file(
			origin: OriginFor<T>,
			cid: T::Hash,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// check if file exists
            ensure!(Files::<T>::contains_key(&cid), Error::<T>::FileDoesNotExist);

			//get the file metadata
			let file2download = <Files<T>>::get(&cid).unwrap();
			let file_size = file2download.file_size;
			let is_allowed = file2download.allow_download;

			//check if file is downloadable
			ensure!(is_allowed, Error::<T>::FileNotDownloadable);


			const ALLOWD_SIZE_FREE:u64 = 250;
			const RATE_PER_UNIT:u64 = 10000000000;



			let cost  = match file2download.file_type {
				// calculate the cost of normal file as per size and free limits. 
				//Also this calculation is not needed if the rate is same as in case of uploading 
				// as cost is already evaluated while uploading and set in price of file
				FileType::Normal => {  
					//calculate the price in u64 
					let cost_in_u64:u64 =  if file_size <= ALLOWD_SIZE_FREE { 
						0 } else { 
							(file_size - ALLOWD_SIZE_FREE) * RATE_PER_UNIT 			
						};
						// convert cost into type Balance
						let new_cost: Option<BalanceOf<T>> = Some(cost_in_u64.saturated_into::<BalanceOf<T>>());
						new_cost
				},
				FileType::Privileged => file2download.price, // keep the cost of the file as it is for privileged file
				
			};

			// // Check the buyer has enough free balance
			ensure!(T::Currency::free_balance(&sender) >= cost.unwrap(), <Error<T>>::NotEnoughBalance);

			// //create dave account
			let dave32 = "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy";
			let dave = T::AccountId::decode(&mut &dave32.as_bytes().encode()[..]).unwrap();


			T::Currency::transfer(&sender, &dave , cost.unwrap(), ExistenceRequirement::KeepAlive)?;

			//increment the download count of individual files
	        <FilesDownloadCnt<T>>::mutate(&cid, |x| {
				let cnt = *x;
				*x = cnt + 1; //Some(cnt + 1);
			});   


			//increment overall file download count
			<TotalDownloadCount<T>>::mutate(|x| *x+=1 );

			//trace downloader details
			//increment the number of downloads by file id and user
			<FileDownloaders<T>>::mutate(&cid, &sender, |x| *x+=1);

			// let an: &[u8] = "5GukQt4gJW2XqzFwmm3RHa7x6sYuVcGhuhz72CN7oiBsgffx".as_bytes();

			let cnt: u64 = <FilesDownloadCnt<T>>::get(&cid); 
            // Deposite file created event
            Self::deposit_event(Event::FileDownloaded{ cid, count: cnt });

	      	Ok(())
	    }

		/// Transfer Ownership .
		#[pallet::weight(100)]
		pub fn change_owner_of_file(
			origin: OriginFor<T>,
			cid: T::Hash,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			//check if file exists
            ensure!(Files::<T>::contains_key(&cid), Error::<T>::FileDoesNotExist);
			//get the owner of the file
			let owner = <FilesOwned<T>>::get(&cid);

			//check if the file is owneed by the sender
			ensure!(owner == core::prelude::v1::Some(sender.clone()), Error::<T>::SenderIsNotOwner);

	        //change the owner of the file in storage
	        <FilesOwned<T>>::mutate(&cid, |_| new_owner);      
            // Deposite file owner changed event
            Self::deposit_event(Event::FileOwnerChanged{ cid, new_owner: sender });
	      		Ok(())
	    }

    }


}