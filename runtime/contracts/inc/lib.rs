#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod inc {

	use flipper::FlipperRef;

	/// Defines the storage of your contract.
	/// Add new fields to the below struct in order
	/// to add new static storage fields to your contract.
	#[ink(storage)]
	pub struct Inc {
		/// Stores a single `bool` value on the storage.
		flipper: FlipperRef,
	}

	impl Inc {
		/// Constructor that initializes the `bool` value to the given `init_value`.
		#[ink(constructor)]
		pub fn new(flipper: FlipperRef) -> Self {
			Self { flipper }
		}

		/// A message that can be called on instantiated contracts.
		/// This one flips the value of the stored `bool` from `true`
		/// to `false` and vice versa.
		#[ink(message)]
		pub fn super_flip(&mut self) {
			// call flipper from inc
			self.flipper.flip();
		}
	}

	/// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
	/// module and test functions are marked with a `#[test]` attribute.
	/// The below code is technically just normal Rust code.
	#[cfg(test)]
	mod tests {}
}
