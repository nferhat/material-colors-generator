use std::{collections::HashMap, fmt::Debug, path::PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use colorsys::{prelude::*, ColorAlpha, Hsl, Rgb};
use image::{imageops::FilterType, GenericImageView, Rgba};
use ini_material_color_utilities_rs::{
    palettes::core::{ColorPalette, CorePalette},
    quantize::quantizer_celebi::QuantizerCelebi,
    scheme::scheme::Scheme,
    score,
};

use anyhow::{Context, Result};

pub trait RgbExt {
    /// Brighten the color.
    ///
    /// What's the different between this and using `Hsl::lighten`?
    ///
    /// Using `Hsl::lighten` causes some slight color deviation towards the hue, I.E if you
    /// darken (by using a negative) amount a color it will make it more saturated towards the
    /// color hsl
    ///
    /// This function doesn't cause this.'
    fn brigthen(&mut self, amount: f64);
}

impl RgbExt for Rgb {
    fn brigthen(&mut self, amount: f64) {
        let (red, green, blue) = (self.red(), self.green(), self.blue());
        self.set_red(red - (255.0 * -(amount / 100.0)).floor());
        self.set_green(green - (255.0 * -(amount / 100.0)).floor());
        self.set_blue(blue - (255.0 * -(amount / 100.0)).floor());
    }
}

#[derive(Debug, Parser)]
#[command(version, long_about = None)]
struct Cli {
    /// The source of the image
    #[command(subcommand)]
    pub source: SchemeSource,

    /// What type of scheme you want to generate?
    #[arg(
        value_enum,
        short,
        long,
        global = true,
        value_name = "TYPE",
        default_value = "dark"
    )]
    pub mode: SchemeMode,

    /// What palette to use when generating colors.
    #[arg(
        value_enum,
        short,
        long,
        global = true,
        value_name = "PALETTTE",
        default_value = "default"
    )]
    pub palette: ColorPalette,
}

#[derive(Clone, Debug, Subcommand)]
enum SchemeSource {
    /// Extract the source color from an image.
    Image { path: PathBuf },
    /// Use this color as the source.
    Color { hex: String },
}

#[derive(Clone, Debug, PartialEq, ValueEnum)]
enum SchemeMode {
    Amoled,
    Dark,
    Light,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let source_color = match cli.source {
        SchemeSource::Image { path } => {
            // Algorithm derived from:
            // * https://github.com/end-4/dots-hyprland/blob/c8f83c2ba329fc426d8f3e439ac9648df7bdf695/.config/ags/scripts/color_generation/generate_colors_material.py#L52
            let image = image::open(path).context("Failed to open image!")?;
            let (width, height) = image.dimensions();
            // Resize the image so that we don't get something huge with a gigantic color
            // variety, causing widely different colorschemes
            let new_width = 64;
            let width_percent = new_width / width;
            let new_height = height * width_percent;
            let resized_image = image.resize(new_width, new_height, FilterType::Lanczos3);

            // Get all the pixels from the resized image then generate a score out of it.
            // in ARGB format.
            let pixels: Vec<[u8; 4]> = resized_image
                .pixels()
                .map(|pixel| {
                    // The Rgba type, in the field index two of pixel, is simply a wrapper around a\
                    // [u8; 4] type, we can simply extract the RGBA data, then turn it into ARGB for
                    // material colors.
                    let Rgba([red, green, blue, alpha]) = pixel.2;
                    [alpha, red, green, blue]
                })
                .collect();

            // Now lend the pixels to material colors.
            let theme = QuantizerCelebi::quantize(&mut QuantizerCelebi, &pixels, 128);
            // The color @ index 0 is basically the one with the highest score, so with the most
            // consistent theme.
            score::score(&theme)[0]
        }
        SchemeSource::Color { hex } => {
            let rgb = Rgb::from_hex_str(&hex).context("Malformed hex code")?;
            [
                rgb.alpha() as u8,
                rgb.red() as u8,
                rgb.green() as u8,
                rgb.blue() as u8,
            ]
        }
    };

    let mut core_palette = CorePalette::new(source_color, true, &cli.palette);
    let scheme = match cli.mode {
        SchemeMode::Amoled => Scheme::pure_dark_from_core_palette(&mut core_palette),
        SchemeMode::Dark => Scheme::dark_from_core_palette(&mut core_palette),
        SchemeMode::Light => Scheme::light_from_core_palette(&mut core_palette),
    };

    // FIXME: Stupid thing but I want a HashMap of everything, not fields of a struct.
    // This will always be a O(n = 46 * 2) operation, though.
    let json_str = serde_json::to_string(&scheme).unwrap();
    let mut colors: HashMap<&str, Rgb> = serde_json::from_str::<HashMap<&str, String>>(&json_str)
        .unwrap()
        .into_iter()
        .map(|(k, v)| (k, Rgb::from_hex_str(&v).unwrap()))
        .collect();

    // TODO: Read post-processing actions from a given file instead of hard coding them.

    // I feel like material UI backgrounds are always too bright?, maybe its me
    // I dim everything beforehand by a constant factor before actual modifications
    for color in [
        "surface",
        "surface_dim",
        "surface_bright",
        "surface_container",
        "surface_container_lowest",
        "surface_container_low",
        "surface_container_high",
        "surface_container_highest",
        "inverse_surface",
        "primary",
        "secondary",
        "tertiary",
        "primary_container",
        "secondary_container",
        "tertiary_container",
        "error",
    ] {
        let mut rgb = colors[color].clone();
        rgb.brigthen(-1.0);
        colors.insert(color, rgb);
    }

    if cli.mode == SchemeMode::Dark {
        // Make surface_dim actually dim, even on dark colorscheme
        let mut rgb = colors["surface_dim"].clone();
        rgb.brigthen(-1.0);
        colors.insert("surface_dim", rgb);
        // And make surface_bright a bit less flaring for dark colorscheme
        // and doing so by deriving it from surface
        let mut hsl: Hsl = colors["surface"].clone().into();
        hsl.lighten(1.35);
        colors.insert("surface_bright", hsl.into());
    }
    if cli.mode == SchemeMode::Light {
        //  make surface_bright actually bright for light colorschemes
        let mut rgb = colors["surface_bright"].clone();
        rgb.brigthen(1.0);
        colors.insert("surface_bright", rgb);
    }

    // Now serialize back and you are done
    let colors: HashMap<&str, String> = colors
        .into_iter()
        .map(|(k, v)| (k, v.to_hex_string().replace("#", "")))
        .collect();
    let json_str = serde_json::to_string(&colors).unwrap();

    println!("{}", json_str);

    Ok(())
}
