# material-colors-generator

Small utility to generate material colors either from a base color or a wallpaper, to be used in my [dotfiles](https://github.com/nferhat/dotfiles)

Outputs in **JSON only**, the point of it is that you parse the json yourself and do whatever with it.

## Installing

1. From source

```sh
cargo build --release
./target/release/material-colors-generator --help
```

2. Nix flake

```nix
{
    inputs = {
        material-colors-generator.url = "github:nferhat/material-colors-generator";
    };

    outputs = inputs: {
        # Now you can use it I guess?
        packages."x86_64-linux".default = inputs.material-colors-generator.packages."x86_64-linux".default;
    };
}
```

## Acknowledgements

1. https://github.com/end-4/dots-hyprland for the idea of using material colors.
2. https://github.com/InioX/matugen for `clap` code to make a usable CLI .
