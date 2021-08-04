#  .08Avif-microservice

This project is a simple POC to wrap AVIF conversion into a web api using wrap.

AVIF is currently (as of 04.08.21) compatible with 67.24% browsers globally.

Polyfill is available at https://github.com/Kagami/avif.js

Current coverage can be checked at https://caniuse.com/avif


## Usage

`cargo run`

make a POST request to http://localhost:3030/upload

## Example

Please check out test-upload.http file in the repo.


## Dependencies

Ravif library - inspired this project
https://github.com/kornelski/cavif-rs/tree/main/ravif

Warp server
https://github.com/seanmonstar/warp/

and a couple of other utility libs.


