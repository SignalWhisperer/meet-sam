# Meet SAM

A basic demonstration of a Rust project using a SAM template to deploy on AWS. There are many workaround required due to issues in the SAM CLI.

## Build

Some workarounds are required to build the project so the SAM CLI can use it correctly. Run the `./build` script to build the project as it is.

## Deploy

Simply run `sam deploy -g` to have a guided deployment, or `sam deploy` if you already have a `samconfig.toml` file created.

