# snowprints-rs

Create unique, sortable ids.

## How to use

To create a `snowprint` use the `compose` function.

```rust
use snowprints::compose;

let snowprint = compose(duration_ms, logical_volume_id, sequence_id);
```

To get values from a `snowprint` use the `decompose` function.

```rust
use snowprints::decompose;

let (timestamp_ms, logical_volume_id, sequence_id) = decompose(snowprint);
```

## Snowprint generation

For a predefined way to cycle through a series of `logical_volumes` and `sequences` use `Snowprints`.

### Settings

First, define a `Settings` struct.

The `logical_volume_base` property defines where to begin logical volume rotations. The `logical_volume_modulo` property defines how many logical volumes will be rotated.

So to rotate through logical volumes `1024-2047` set `logical_volume_base` to `1024` and `logical_volume_modulo` to `1024`.

In the example below, a `Snowprint` called `snowprinter` will track milliseconds since `2024 Jan 1st` and rotate through logical volumes `0-8191`.

```rust
use std::time::Duration;
use snowprints::{Settings, Snowprint};

let settings = Settings {
    origin_system_time: UNIX_EPOCH + Duration::from_millis(EPOCH_2024_01_01_AS_MS),
    logical_volume_base: 0,
    logical_volume_modulo: 8192,
};

let mut snowprinter = match Snowprint::new(settings) {
    Ok(snow) => snow,
    Err(err) => return println!("settings might be bad: {}", err.to_string()),
};
```

The function `snowprinter.compose()` will only error when available `logical_volumes` and `sequences` have been exhausted for the current `millisecond`.

```rust
use snowprints::decompose;

let snowprint = match snowprinter.compose() {
    Ok(sp) => sp,
    Err(err) => return println!("ran out of sequences and ids!: {}", err.to_string()),
};

let (timestamp_ms, logical_volume_id, sequence_id) = decompose(snowprint);
```

## What is a snowprint?

A snowprint is an alternative to unique id generation patterns like `snowflakes`.
A `snowprint` is defined by bitshifting the following values into a 64bit unsigned integer:
- 41bit `ms duration` since an arbitrary date in millseconds
- 13bit `logical_volume` between `0-8191`
- 10bit `sequence` between `0-1023`.

## License

`Snowprint` is released under the BSD 3-Clause License