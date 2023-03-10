## Kaizen

Solana OS Rust framework for industrial grade applications.

[<img alt="github" src="https://img.shields.io/badge/github-solana--kaizen/kaizen-8da0cb?style=for-the-badge&labelColor=555555&color=8da0cb&logo=github" height="20">](https://github.com/solana-kaizen/kaizen)
[<img alt="crates.io" src="https://img.shields.io/crates/v/kaizen.svg?maxAge=2592000&style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/kaizen)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-kaizen-56c2a5?maxAge=2592000&style=for-the-badge&logo=docs.rs" height="20">](https://docs.rs/kaizen)
<img alt="license" src="https://img.shields.io/crates/l/kaizen.svg?maxAge=2592000&color=6ac&style=for-the-badge&logoColor=fff" height="20">
<img src="https://img.shields.io/badge/platform-native-informational?style=for-the-badge&color=50a0f0" height="20">
<img src="https://img.shields.io/badge/platform-wasm32/browser-informational?style=for-the-badge&color=50a0f0" height="20">
<img src="https://img.shields.io/badge/platform-wasm32/node.js-informational?style=for-the-badge&color=50a0f0" height="20">
<img src="https://img.shields.io/badge/platform-solana_os-informational?style=for-the-badge&color=50a0f0" height="20">

<p align="center" style="margin:32px auto 32px auto;text-align:center;font-size:10px;color:#888;">
<img src="https://upload.wikimedia.org/wikipedia/commons/thumb/a/a5/Tsunami_by_hokusai_19th_century.jpg/2560px-Tsunami_by_hokusai_19th_century.jpg" style="display:block;height:320px;width:auto;margin: 0px auto 0px auto;"><br/><sup>THE GREAT WAVE OFF KANAGAWA &bull; KATSUSHIKA HOKUSAI &bull; JAPAN 1831</sup></p>

‘Kaizen’ focuses on the refinement of the Solana application development infrastructure by identifying framework optimization opportunities in order to increase reliability and simplify Solana-based full-stack application development. 

## Overview

Kaizen is a *security-and-reliability-centric* framework for developing of Solana Programs and *client-side web applications* using Rust. The primary goal behind this project is to eliminate IDLs and contain the program and client-side application within the same Rust codebase.

This in-turn allows developers to use functions and data structures that are a part of the program directly within the client-side web application.

The framework is backed by native and in-browser async Rust transport layers that can fetch account data and access it client-side via functions interfacing with [AccountInfo](https://docs.rs/solana-program/latest/solana_program/account_info/struct.AccountInfo.html) and *Account Data Containers*.

An example is available here: <https://github.com/solana-kaizen/kaizen-example>


## Features

* Unified async Rust Web3 transport interface (uses native Rust Solana implementation when building native and Web3.js implementation when running under WASM32 browser environment).
* Built on top of [`workflow-rs`](https://github.com/workflow-rs/workflow-rs) async Rust application development framework and designed to provide unified environment where functions can be used client-side (for example, a function using `workflow_log::log_info!()` will invoke printf!() on native, `console.log()` in browser and `solana_program::log::sol_log()` under Solana OS).
* Unified Solana *instruction builder interface* that uses [Rust Builder Pattern](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html) and includes various functionality for account data exchange and PDA key management. The instruction builder supports creation of batch transactions for dispatch of multi-stage (or large-side) operations and includes transaction queue management functionality.
* Macros for program function mappings, allowing invocation of program functions by function names in-client.
* Segmented account data storage derived from Rust structure declarations, allowing each structure field to be accessed directly, and resized. Segments can be memory-mapped data structures as well as borsh-serialized data structures.
* Container-based approach for account management with a simultaneous in-program and client-side container type registration.
* Client-side container access and caching mechanisms (using async Rust transport api).
* Solana Emulator (extremely simplified) provides the developer with the ability to run programs on native targets (OS) and in-browser (in WASM). This emulator supports a limited subset of account functionality such as account resizing, SOL balance tracking etc., and is intended for testing and prototyping program functionality in the native of in-browser environments. The emulator also supports basic scaffolding for off-chain program unit-testing.
* async Rust subsystem for client-side account data and container fetching, including application-level in-memory account data cache.
* Basic user identity data structures allowing multiple wallets to be bound to a single user identity.
* `Instant` data structure for time tracking (uses Instance on native, `Date::now()` in WASM32 and `Clock::get()` in Solana).
* Support for integration with multiple Solana Programs as well as interfacing with multiple programs from within a single application.
* Helper functions for automated account creation and resizing.

## Motivation

- Interpreted languages such as TypeScript and JavaScript are inherently unsecure, especially taking into account package managers such as NPM and general practices of developers using them. There are various code-injection attacks that can be performed on the code written in these languages. These technologies should not be used in high-security and high-reliability applications, especially business oriented cryptocurrency applications. Rust + WASM greatly reduces these attack surfaces.
- Solana application frameworks such as [Anchor](https://www.anchor-lang.com/) rely on exposing data structures via IDL, introducing multiple layers of tools and technologies between the application and the program. Rust compiled straight into WASM eliminates these layers, allowing application developer to publish primitives directly from the Rust codebase into front-end applications. In many cases, the core application functionality can be written in Rust exposing only API calls needed by the application front-end, thus imposing Rust reliability and strict type system onto the core of the web application.
- When creating complex APIs meant to interface with Solana programs, at times it is desirable to create both a web front-end and a server backend that are capable of communicating with the network and on-chain programs. APIs developed on top of Kaizen, function uniformly in native applications and in web applications. Currently, to function in web applications and to interface with wallets, Kaizen uses Solana web3 libraries. It is our goal in the long-term to completely eliminate web3 dependencies.

## Development status

We have been using the framework for in-house development for few months, gradually improving it.  The framework is currently under developmnet and should be considered in alpha / preview stage. Additional work is needed on documentation and tutorials.  We are looking for sponsorship to help us dedicate more time toward polishing this platform.

You should currently approach this only if you are confident in your Rust skills, have good understanding of the Solana Account system and are looking to develop large-scale business or "industrial-grade" applications exposing WASM APIs client-side.

If you would like to develop applications using this project, we can help answer any questions and guide you through the APIs.  Join us on ASPECTRON Discord server if you have any questions. You can find the link for Discord at [https://aspectron.com/en/index.html#about](https://aspectron.com/en/index.html#about)

## TODO

- Parallelism in collection account creation - Currently, in a multi-user environment, functions that provide multiple users with an ability to create collection-bound accounts (PDAs), can collide if multiple users execute account creation in parallel.  To mitigate this, the collection creation functionality needs to supply a list of accounts, while the program API handling account creation needs to select corresponding account templates based on the cursor (collection length) value at the moment of the program execution.
- Support for WebSocket updates - Kaizen does not currently support any type of network-side event updates. We need to implement program monitoring channels and create bindings to the Transport and Transaction Queue to automate processes like account creation notifications.
- Refactor Kaizen WASM APIs to use [`Sendable<T>()`](https://github.com/workflow-rs/workflow-rs/blob/master/wasm/src/sendable.rs) wrappers - to date, we have been using `#[async_trait]` and `#[async_trait(?Send)]` macros that were re-exported by the [`workflow-async-trait`](https://github.com/workflow-rs/workflow-async-trait) crate as `#[workflow_async_trait]` where the Send marker would be required on the async trait in the Rust native environment (so that it can be used under *Tokio*) and not required in WASM32 environment (so that it can be used under *async_std*).  After using the framework extensively we have concluded that using `Sendable<T>` wrappers is much more efficient and cleaner, removing the need for any *async_trait* customizations.
- Integrate basic wallet functionality and a wallet API as there are use-cases where it may be desirable for business applications to include their own in-application wallets to automate payments. While using web apps in a browser environment, user can take advantage of the browser-compatible wallets (such as Phantom), in native Rust environment, user can utilize native commant-line wallet.  However, Kaizen, combined with [NWJS](https://nwjs.io) backed by [`workflow-nw`](https://crates.io/crates/workflow-nw) crate, combined with [`cargo-nw`](https://aspectron.com/en/projects/cargo-nw.html) redistributable package builder, it is possible to create fully-featured HTML-powerd traditional desktop applications installable in Windows, MacOS and Linux environments.  However, such applications currently lack the ability to have an interactive wallet (although NWJS supports chrome extensions and technically it should be possible to install Phantom within NWJS, but such installation will be rather complex for the end-user and play against shipping a fully-integrated easy-to-use product).
- Review the entire framework to see which components can be isolated into Rust crate features in an effort to see if we can reduce the footprint of the resulting SBF bytecode.