# Simple Rust Raytracer

![Latest Render](./image.png?raw=true)

> Simple multithreaded raytracer written in rust, able to load simple json map files and render them.

## Build
Building in release mode improves performance by a major factor. To build this renderer run.
```
git clone https://gitlab.com/vgarleanu/raytracer
cargo build --release
```
To run, either use `cargo --run` or `target/debug/raytracer`.

## Example
To render the basic built in demo map just run
This command will render the default map with 400 rays per pixel at a resolution of 400x400, and will use 16 threads:
```
raytracer -o image_file.jpg --threads 16 -x 400 -y 400 -r 400
```
To load a custom map run:
```
raytracer -m ./map.json -o image_file.jpg
```
For more options run
```
raytracer -h
```

## Contributing
Contributions are absolutely, positively welcome and encouraged! Contributions
come in many forms. You could:

  1. Submit a feature request or bug report as an [issue].
  2. Ask for improved documentation as an [issue].
  3. Contribute code via [merge requests].

[issue]: https://gitlab.com/vgarleanu/raytracer/issues
[merge requests]: https://gitlab.com/vgarleanu/raytracer/merge_requests

All pull requests are code reviewed and tested by the CI. Note that unless you
explicitly state otherwise, any contribution intentionally submitted for
inclusion in this software by you shall be licensed under the MIT License 
without any additional terms or conditions.

## License
This software is licensed under the MIT license ([LICENSE.md](LICENSE.md) or http://opensource.org/licenses/MIT)
