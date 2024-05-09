# snowprints-rs

Create unique and sortable ids.

## What is a snowprint?

A snowprint unique id generation pattern defined by bitshifting the following values into a 64bit unsigned integer:
- 41 bit `duration` since an arbitrary date in millseconds
- 13 bit `logical_volume` between `0-8191`
- 10 bit `sequence` between `0-1023`.

`snowprint` is based rougly on this [article](https://instagram-engineering.com/sharding-ids-at-instagram-1cf5a71e5a5c).

## How to use

To create a `snowprint` use the `compose` function.

```rust
use snowprints::compose;

let snowprint = compose(duration_ms, logical_volume, sequence);
```

To get values from a `snowprint` use the `decompose` function.

```rust
use snowprints::decompose;

let (timestamp_ms, logical_volume, sequence) = decompose(snowprint);
```

## Snowprint generation

For a predefined way to cycle through a series of `logical_volumes` and `sequences` use `Snowprints`.

### Settings

First, define a `Settings` struct.

The `logical_volume_base` property defines where to begin logical volume rotations. The `logical_volume_length` property defines how many logical volumes will be rotated.

To rotate through logical volumes `1024-2047`, set `logical_volume_base` to `1024` and `logical_volume_length` to `1024`.

```rust
use std::time::Duration;
use snowprints::Settings;

let settings = Settings {
    origin_system_time: UNIX_EPOCH + Duration::from_millis(EPOCH_2024_01_01_AS_MS),
    logical_volume_base: 0,
    logical_volume_length: 8192,
};

```

### Compose snowprints

In the example below, a `Snowprint` called `snowprinter` will track milliseconds since `2024 Jan 1st` and rotate through logical volumes `0-8191`.

```rust
use snowprints::Snowprint;
use snowprints::decompose;

let mut snowprinter = match Snowprint::new(settings) {
    Ok(snow) => snow,
    _ => return println!("Settings are not valid!"),
};

let snowprint = match snowprinter.compose() {
    Ok(sp) => sp,
    _ => return println!("Consumed all available logical volumes and sequences!"),
};

let (timestamp_ms, logical_volume, sequence) = decompose(snowprint);
```

The function `snowprinter.compose()` will only error when available `logical_volumes` and `sequences` have been exhausted for the current `millisecond`.

## Why can't I choose my own bit lengths?

A `snowprint` is a unique identifier meant to last up to `41 years`. The ids will most likely outlive the code, organization, or even the author that generated them.

If bit lengths are available as an API, a developer will inevitably change them and cause immense and incalculable pain for whatever unlucky system in that 41 year time period.

If a custom set of bit lengths are neccessary, fork this repo and change the following values:

```rust
// ./src/lib.rs
const SEQUENCE_BIT_LEN: u64 = 10;
const LOGICAL_VOLUME_BIT_LEN: u64 = 13;
```

## License

`Snowprints` is released under the BSD 3-Clause License