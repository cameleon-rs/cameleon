# Examples

## [stream.rs](stream.rs)
Describes how to start streaming and reeceive payloads.

```sh
cargo run --example stream --features=libusb
```

## [params.rs](params.rs)
Describes how to configure parameters of a camera.

```sh
cargo run --example params --features=libusb
```

## [no_cache.rs](no_cache.rs)
Describes how to use internal type conversions in `Camera`.

See also [custom_ctxt.rs](custom_ctxt.rs) that describes the more advanced use of type conversions

```sh
cargo run --example no_cache --features=libusb
```

## [custom_ctxt.rs](custom_ctxt.rs)
Describes how to define custom `GenApi` context.

In this example, we'll define a context in which the cache can be dynamically switched on and off.

```sh
cargo run --example custom_ctxt --features=libusb
```

## [u3v](u3v)
Describes how to manipulate `USB3 vision` camera's specific features.
