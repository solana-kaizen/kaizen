## Kaizen

[<img alt="github" src="https://img.shields.io/badge/github-solana--kaizen/kaizen-8da0cb?style=for-the-badge&labelColor=555555&color=8da0cb&logo=github" height="20">](https://github.com/solana-kaizen/kaizen)
[<img alt="crates.io" src="https://img.shields.io/crates/v/kaizen.svg?maxAge=2592000&style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/kaizen)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-kaizen-56c2a5?maxAge=2592000&style=for-the-badge&logo=rust" height="20">](https://docs.rs/kaizen)
<img alt="license" src="https://img.shields.io/crates/l/kaizen.svg?maxAge=2592000&color=6ac&style=for-the-badge&logoColor=fff" height="20">
<img src="https://img.shields.io/badge/platform-native-informational?style=for-the-badge&color=50a0f0" height="20">
<img src="https://img.shields.io/badge/platform-wasm32/browser-informational?style=for-the-badge&color=50a0f0" height="20">
<img src="https://img.shields.io/badge/platform-wasm32/node.js-informational?style=for-the-badge&color=50a0f0" height="20">
<img src="https://img.shields.io/badge/platform-solana_os-informational?style=for-the-badge&color=50a0f0" height="20">

## Overview

This crate is a *security-and-reliability-centric* framework for development of Solana Programs using Rust and *client-side applications using pure async Rust*. The primary goal behind this project is to eliminate IDLs and contain the program and client-side application within the same Rust codebase, allowing program functionaity, if needed, to exist in the same Rust file as the client-side functionality.

This in-turn allows create of a single data processing layer that is able to process account data in-program as well as client-side.

The framework is then backed by native and in-browser async Rust transport layers that can fetch account data and access it client-side via functions interfacing with [AccountInfo](https://docs.rs/solana-program/latest/solana_program/account_info/struct.AccountInfo.html).

Example available here: <https://github.com/solana-kaizen/kaizen-example>

## Features

* Unified async Rust Web3 transport interface (uses native Rust Solana implementation when building native and Web3.js implementation when running under WASM32 browser environment).
* Built on top of [`workflow-rs`](https://github.com/workflow-rs/workflow-rs) - an async Rust application development framework and designed to provide unified environment where Solana Program functions can be used client-side (for example, a function using `workflow_log::log_info!()` will invoke printf!() on native, `console.log()` in browser and `solana_program::log::sol_log()` under Solana OS).
* Unified Solana Instruction builder interface that uses [Rust Builder Pattern](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html) and includes various framework-centric functionality.
* Macros for program function mappings, allowing invocation of program functions by function names in-client. 
* Segmented account data storage derived from Rust structure declarations, allowing each structure field to be accessed directly, and resized. Segments can be memory-mapped data structures as well as borsh-serialized data structures.
* Container-based approach for account management with a simultaneous in-program and client-side container type registration.
* Client-side container access and caching mechanisms (using async Rust transport api).
* Solana Emulator (extremely simplified) provides the developer with the ability to run programs on native targets (OS) and in-browser (in WASM). This emulator supports a limited subset of account functionality such as account resizing, SOL balance tracking etc., and is intended for testing and prototyping program functionality in the native of in-browser environments.
* Basic user identity data structures allowing multiple wallets to be bound to a single user identity.
* `Instant` data structure for time tracking (uses Instance on native, `Date::now()` in WASM32 and `Clock::get()` in Solana).
* Support for integration with multiple Solana Programs as well as interfacing with multiple programs from within a single application.
* Helper functions for automated account creation and resizing.

