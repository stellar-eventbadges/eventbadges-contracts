#![no_std]

// Keep this file thin: `#[contract]` and `#[contractimpl]` only. Logic,
// types, and storage rules live in the modules below.
mod types;

use soroban_sdk::{contract, contractimpl, Env};

pub use crate::types::{Badge, Error, Event};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {}
