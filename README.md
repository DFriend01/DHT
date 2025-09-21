# DHT

## Setup

1. Install [rust](https://doc.rust-lang.org/book/ch01-01-installation.html)

2. Install the [protobuf compiler](https://protobuf.dev/installation/)

## Run the tests

The [scripts](./dht/scripts/) directory has utility scripts to help with running tests:

- Run the unit tests: `./run_unit_tests.sh`
- Run the integration tests: `./run_single_node_integration_tests.sh`

The integration test script should stop all servers once the tests are complete. If something goes
wrong, `stop_servers.sh` can be used to stop the server instances.
