// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Traits used across pallets for Polkadot.

use primitives::v1::Id as ParaId;

use frame_support::{
	dispatch::DispatchResult,
	traits::{Currency, ReservableCurrency},
};

use sp_std::vec::*;

/// Parachain registration API.
pub trait Registrar {
	/// All parachains. Ordered ascending by ParaId. Parathreads are not included.
	fn parachains() -> Vec<ParaId>;

	/// Return if a ParaId is a Parachain.
	fn is_parachain(id: ParaId) -> bool {
		Self::parachains().binary_search(&id).is_ok()
	}

	/// Return if a ParaId is a Parathread.
	fn is_parathread(id: ParaId) -> bool;

	/// Return if a ParaId is registered in the system.
	fn is_registered(id: ParaId) -> bool {
		Self::is_parathread(id) || Self::is_parachain(id)
	}

	/// Elevate a para to parachain status.
	fn make_parachain(id: ParaId) -> DispatchResult;

	/// Lower a para back to normal from parachain status.
	fn make_parathread(id: ParaId) -> DispatchResult;
}

/// Error type for something that went wrong with leasing.
pub enum LeaseError {
	/// Unable to reserve the funds in the leaser's account.
	ReserveFailed,
	/// There is already a lease on at least one period for the given para.
	AlreadyLeased,
	/// The period to be leased has already ended.
	AlreadyEnded,
}

/// Lease manager. Used by the auction module to handle parachain slot leases.
pub trait Leaser {
	/// An account identifier for a leaser.
	type AccountId;

	/// The measurement type for counting lease periods (generally just a `BlockNumber`).
	type LeasePeriod;

	/// The currency type in which the lease is taken.
	type Currency: ReservableCurrency<Self::AccountId>;

	/// Lease a new parachain slot for `para`.
	///
	/// `leaser` shall have a total of `amount` balance reserved by the implementor of this trait.
	///
	/// Note: The implementor of the trait (the leasing system) is expected to do all reserve/unreserve calls. The
	/// caller of this trait *SHOULD NOT* pre-reserve the deposit (though should ensure that it is reservable).
	///
	/// The lease will last from `period_begin` for `period_count` lease periods. It is undefined if the `para`
	/// already has a slot leased during those periods.
	///
	/// Returns `Err` in the case of an error, and in which case nothing is changed.
	fn lease_out(
		para: ParaId,
		leaser: &Self::AccountId,
		amount: <Self::Currency as Currency<Self::AccountId>>::Balance,
		period_begin: Self::LeasePeriod,
		period_count: Self::LeasePeriod,
	) -> Result<(), LeaseError>;

	/// Return the amount of balance currently held in reserve on `leaser`'s account for leasing `para`. This won't
	/// go down outside of a lease period.
	fn deposit_held(para: ParaId, leaser: &Self::AccountId) -> <Self::Currency as Currency<Self::AccountId>>::Balance;

	/// The lease period. This is constant, but can't be a `const` due to it being a runtime configurable quantity.
	fn lease_period() -> Self::LeasePeriod;

	/// Returns the current lease period.
	fn lease_period_index() -> Self::LeasePeriod;
}

/// Runtime hook for when we swap a parachain and parathread.
pub trait OnSwap {
	/// Updates any needed state/references to enact a logical swap of two parachains. Identity,
	/// code and `head_data` remain equivalent for all parachains/threads, however other properties
	/// such as leases, deposits held and thread/chain nature are swapped.
	fn on_swap(one: ParaId, other: ParaId);
}

impl OnSwap for () {
	fn on_swap(_: ParaId, _: ParaId) {}
}
