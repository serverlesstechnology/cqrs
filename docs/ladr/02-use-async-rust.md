# Use async Rust

* Status: accepted
* Date: 2021-06-02

## Context

As part of 
[release 1.39.0](https://blog.rust-lang.org/2019/11/07/Rust-1.39.0.html)
Rust began support for async-await. 
Since then a significant number of libraries have moved to async making their use difficult from within the `cqrs-es`
library 
([red/blue functions](https://journal.stuffwithstuff.com/2015/02/01/what-color-is-your-function/)). 

## Decision Drivers

- A significant number of libraries have moved to use async.
- Most open source server frameworks now use async.  

## Considered options

- Move the library to async-await.
  - Greatly simplifies the use of libraries built on async, in particular those using http clients.
  - Better suited for use in web applications.
- Continue using standard Rust.
  - Using async requires a runtime which could increase application start time. This both adds call latency and 
increases the risk of serverless cold start timeouts.
  - Core Rust features such as traits do not yet support async.

## Decision outcome

Use async with the Tokio runtime:
- The additional work to use asynchronous libraries without async is significant and more than can be justified.
- Many database drivers (core dependencies of the persistence packages) now are async.
- Absence of async severely limits the HTTP servers that can be used for standalone applications.

