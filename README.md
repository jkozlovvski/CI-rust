# Simple continuous Integration System

## Authors
- Julian Kozlowski (@Yashiru99 on GitHub)

## Description
The project is based on `https://aosabook.org/en/500L/a-continuous-integration-system.html`.
It will be simple CLI system, it will be based on 3 components: an observer, a test job dispatcher, and a test runner. They are 3 separate processes and will communicate to each other via websockets.

## Features
- Running tests on the recent commit after 5 seconds period
- Gathering files of test results in the file system local to the dispatcher

## Plan
As the components are complementary to each other (via communicating) it's tough to make distinction into two separate parts, but if I were to I would do it like that:

1st part: Implementing observer and dispatcher
2nd part: Implementing test-runner and tests (and CI-runner to run all services at once)

## Libraries
serde, log

## Potential todo's and refactors left from part 1:
- use tokio in the dispatcher service instead of periodically checking
- use `clap` for command line parsing
- make erros more robust using `anyhow` and `thiserror` 
- (?) potentially change structure of project
- (?) move all variables to local envs instead of parsing them command line

## How to run the certain binaries
First change local_envs file to appropriate paths on your machine, then source local_envs, then you can run specified binary in this manner:

```rust
cargo run --bin repo_observer
cargo run --bin dispatcher
```
If you want to print debug messages, procede cargo command with `RUST_LOG=debug`.