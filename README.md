# plebscript

plebscript is a toy reimplementation of the API from the now defunct <https://webscript.io> on top of Fastly's compute platform.

It currently supports only the basic request and response parameters (except `form` and `files` in request).

It is built using [`Piccolo`](), which is also, itself, incomplete.

## Usage

See <https://www.fastly.com/documentation/guides/compute/> for initial setup, stop before you get to `fastly compute init`.

Put whatever scripts you'd like in `www/src/` and  run `fastly compute publish --service-id=<YOUR SERVICE ID>` from the `fastly` directory.

