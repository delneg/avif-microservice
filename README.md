#  Avif-microservice

This project is a simple POC to wrap AVIF conversion into a web api using wrap.

AVIF is currently (as of 04.08.21) compatible with 67.24% browsers globally.

Polyfill is available at https://github.com/Kagami/avif.js

Current coverage can be checked at https://caniuse.com/avif


## Usage

`cargo run`

Navigate to `http://localhost:3030` or make a POST request with multipart formdata with 'file' field directly to /upload


Tip: to enable AVIF in firefox, type `about:config` in address bar, and then search for 'avif' to toggle it.


## Example

Please check out test-upload.http file in the repo.

testimg2.jpg (840x770 jpeg 24-bit color 256.72 KB) -> Success: 92KB (91640B color, 0B alpha, 238B HEIF)
testimg.png (840x770 png 32-bit color 481.83 KB) -> Success: 90KB (89644B color, 0B alpha, 238B HEIF)

## Dependencies

Ravif library - inspired this project
https://github.com/kornelski/cavif-rs/tree/main/ravif

Warp server
https://github.com/seanmonstar/warp/

and a couple of other utility libs.


