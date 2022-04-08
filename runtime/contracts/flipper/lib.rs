#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

pub use flipper::*;

#[ink::contract]
mod flipper {

	/// Defines the storage of your contract.
	/// Add new fields to the below struct in order
	/// to add new static storage fields to your contract.
	#[ink(storage)]
	#[derive(Default)]
	pub struct Flipper {
		/// Stores a single `bool` value on the storage.
		value: bool,
	}

	impl Flipper {
		/// Constructor that initializes the `bool` value to the given `init_value`.

		/// Constructor that initializes the `bool` value to `false`.
		///
		/// Constructors can delegate to other constructors.
		#[ink(constructor)]
		pub fn default() -> Self {
			Default::default()
		}

		/// A message that can be called on instantiated contracts.
		/// This one flips the value of the stored `bool` from `true`
		/// to `false` and vice versa.
		#[ink(message, selector = 0xcde4efa9)]
		pub fn flip(&mut self) {
			self.value = !self.value;
		}

		/// Simply returns the current value of our `bool`.
		#[ink(message)]
		pub fn get(&self) -> bool {
			self.value
		}
	}
}
