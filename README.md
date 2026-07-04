# bbmodel-rs

A Rust library for parsing and serializing [Blockbench](https://www.blockbench.net/) model files (`.bbmodel`).

## Features

- **Deserialize & serialize** `.bbmodel` files via [serde](https://serde.rs)
- Covers cubes, meshes, groups, outliner hierarchy, textures, animations, keyframes, display settings, and collections
- Optional [glam](https://github.com/bitshifter/glam-rs) integration - use `glam::Vec3`/`Vec2`/`Vec4` or plain `[f32; N]` arrays
- `no_std` compatible (disable the `std` feature)

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
bbmodel-rs = "0.1"
```

### Deserialize a `.bbmodel` file

```rust
use bbmodel_rs::BBModel;

let file = std::fs::read_to_string("model.bbmodel")?;
let model: BBModel = serde_json::from_str(&file)?;

println!("Model: {:?}", model.name);
println!("Elements: {}", model.elements.len());
```

### Serialize back to JSON

```rust
let json = serde_json::to_string_pretty(&model)?;
std::fs::write("output.bbmodel", json)?;
```

### With glam types

Enable the `glam` feature to use `glam::Vec3`, `glam::Vec2`, and `glam::Vec4` instead of raw float arrays:

```toml
[dependencies]
bbmodel-rs = { version = "0.1", features = ["glam"] }
```

### `no_std` support

```toml
[dependencies]
bbmodel-rs = { version = "0.1", default-features = false }
```

## License

MIT
