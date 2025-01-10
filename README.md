# rustbrush
Painting app and utilities in Rust. Uses `pixels`, `wgpu` and `winit`.

![image](https://github.com/user-attachments/assets/6f225fba-e16f-467a-ab87-8d1ab330ab8c)

## !!!A work in progress!!!
This project is still in development, and as it stands it's not much at all. The goal is to create a simple painting app that can be used to draw simple images. At this point it is highly experimental as I've never done something like this before.

## Usage
You can test the app by running the following command:
```bash
cargo run --release
```

For testing the end user experience, it's recommended that you run in release mode as the debug mode is very slow.

## Contributing
Contributions are welcome! Feel free to open an issue or a pull request.

### Workspace Structure
This workspace is divided into the following crates:
- `rustbrush_app`: The main application crate.
- `rustbrush_utils`: This is where brush operations are defined. I'd like to keep this separate as much as possible, for anyone who wants to create a painting app themselves, they may just want some small subset of operations. It's also just generally a good separation of concerns imo.

## License
MIT

