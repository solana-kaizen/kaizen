## Kaizen

[![Crates.io](https://img.shields.io/crates/l/kaizen.svg?maxAge=2592000)](https://crates.io/crates/kaizen)
[![Crates.io](https://img.shields.io/crates/v/kaizen.svg?maxAge=2592000)](https://crates.io/crates/kaizen)
![platform](https://img.shields.io/badge/platform-Native-informational)
![platform](https://img.shields.io/badge/platform-Web%20%28wasm32%29-informational)
![platform](https://img.shields.io/badge/platform-BPF-informational)

## Overview

This crate is a **security-centric** framework for development of Solana Programs using Rust and **client-side applications using pure async Rust**. The primary goal behind this project is to eliminate IDLs and contain the program and client-side application within the same Rust codebase, allowing program functionaity, if needed, to exist in the same Rust file as the client-side functionality.

This in-turn allows create of a single data processing layer that is able to process account data in-program as well as client-side.

The framework is then backed by native and in-browser async Rust transport layers that can fetch account data and access it client-side via functions interfacing with [AccountInfo](https://docs.rs/solana-program/latest/solana_program/account_info/struct.AccountInfo.html).

## Features

* Unified async Rust Web3 transport interface (uses native Rust Solana implementation when building native and Web3.js implementation when running under WASM32 browser environment)
* Designed to provide unified environment where Solana Program functions can be used client-side (for example, a function using `workflow-log::log_info!()` will invoke printf!() on native, `console.log()` in browser and `solana_program::log::sol_log()` under BPF)
* Unified Solana Instruction builder interface that uses [Rust Builder Pattern](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html) and includes various framework-centric functionality
* Macros for program function mappings, allowing invocation of program functions by function names in-client. 
* Segmented account data storage derived from Rust structure declarations, allowing each structure field to be accessed directly, and resized.
* Container-based approach for account management with a simultaneous in-program and client-side container type registration.
* Client-side container access and caching mechanisms (using async Rust transport api)
* Solana Emulator (extremely simplified) provides the developer with the ability to run programs on native targets (OS) and in-browser (in WASM). This emulator supports a limited subset of account functionality such as account resizing, SOL balance tracking etc., and is intended for testing and prototyping program functionality in the native of in-browser environments.
* Basic user identity data structures allowing multiple wallets to be bound to a single user identity.
* Instant data structure for time tracking (uses Instance on native, `Date::now()` in WASM32 and `Clock::get()` in Solana)
* Support for integration with multiple Solana Programs as well as interfacing with multiple programs from within a single application.
* Helper functions for automated account creation and resizing.

