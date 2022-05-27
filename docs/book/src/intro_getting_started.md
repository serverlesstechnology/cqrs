## Getting started

For this tutorial we will build an application to manage the logic of a bank account.
As a simple set of business rules, we want to:

- accept deposits
- provide withdrawals
- allow our customers to write checks
- disallow customers from overdrawing their account

### Project setup

Okay let's get some code going. First, up we need a workspace. 
You know the drill, find your favorite playspace on your hard 
drive and start a new Rust bin project.

    cargo new --bin mybank

There is a lot that happens behind the scenes in a CQRS/event sourcing application, so we'll be using the 
[`cqrs-es`](https://docs.rs/cqrs-es) framework to get us off the ground.    
Add these dependencies in your cargo.toml:

```toml
[dependencies]
cqrs-es = "0.3.0"
async-trait = "0.1.52"
serde = { version = "1.0", features = ["derive"]}
tokio = { version = "1", features = ["full"] }
```
> All of the examples included here are simplified from the 
> [cqrs-demo](https://github.com/serverlesstechnology/cqrs-demo) project.
> More detailed examples can be found by exploring that package.


