use leptos::{html::Input, logging::log, prelude::*, task::spawn_local};
use web_sys::{js_sys, wasm_bindgen::JsCast, HtmlInputElement};
use image::{DynamicImage, ImageBuffer, ImageFormat, RgbImage, GrayImage};
use std::io::Cursor;
use base64::{engine::general_purpose, Engine as _};
// use wasm_bindgen::prelude::*;
use ndarray::{Array2, Array3, s};
use nshare::{self, AsNdarray3};


async fn convert_image_input_to_base_64(input: Option<HtmlInputElement>) -> Result<String, String> {
    let input = input.ok_or("No input element found")?;
    let files = input.files().ok_or("No files selected")?;
    let file = files.get(0).ok_or("No file found")?;

    // Read file as ArrayBuffer
    let array_buffer_promise = file.array_buffer();
    let array_buffer = wasm_bindgen_futures::JsFuture::from(array_buffer_promise)
        .await
        .map_err(|e| format!("Failed to read file: {:?}", e))?;

    // Convert to Uint8Array and then to Vec<u8>
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let buffer_vec = uint8_array.to_vec();

    // Decode the image
    let img = image::load_from_memory(&buffer_vec)
        .map_err(|e| format!("Failed to decode image: {:?}", e))?;
    let img: RgbImage = img.into_rgb8();

    // Convert original image to base64 for display
    let mut original_buffer = Vec::new();
    img.write_to(&mut Cursor::new(&mut original_buffer), ImageFormat::Png)
        .map_err(|e| format!("Failed to encode original image: {:?}", e))?;
    let original_base64 = general_purpose::STANDARD.encode(&original_buffer);
    Ok(format!("data:image/png;base64,{}", original_base64))
}


async fn process_image(input: Option<HtmlInputElement>) -> Result<(String, String), String> {
    let input = input.ok_or("No input element found")?;
    let files = input.files().ok_or("No files selected")?;
    let file = files.get(0).ok_or("No file found")?;

    // Read file as ArrayBuffer
    let array_buffer_promise = file.array_buffer();
    let array_buffer = wasm_bindgen_futures::JsFuture::from(array_buffer_promise)
        .await
        .map_err(|e| format!("Failed to read file: {:?}", e))?;

    // Convert to Uint8Array and then to Vec<u8>
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let buffer_vec = uint8_array.to_vec();

    // Decode the image
    let img = image::load_from_memory(&buffer_vec)
        .map_err(|e| format!("Failed to decode image: {:?}", e))?;
    let img: RgbImage = img.into_rgb8();

    // Convert original image to base64 for display
    let mut original_buffer = Vec::new();
    img.write_to(&mut Cursor::new(&mut original_buffer), ImageFormat::Png)
        .map_err(|e| format!("Failed to encode original image: {:?}", e))?;
    let original_base64 = general_purpose::STANDARD.encode(&original_buffer);
    let original_data_url = format!("data:image/png;base64,{}", original_base64);

    // Process the image with TGV denoising
    // let rgb_img: RgbImage = img.to_rgb8();
    let img = img.as_ndarray3();

    // Quantize colors to 6^3 colors
    let mut quantized_img = Array2::<u8>::zeros((img.dim().0, img.dim().1));
    let num_colors_per_channel = 5;
    for i in 0..img.dim().0 {
        for j in 0..img.dim().1 {
            let r_pixel = img.get((i, j, 0)).unwrap() / num_colors_per_channel;
            let g_pixel = img.get((i, j, 1)).unwrap() / num_colors_per_channel;
            let b_pixel = img.get((i, j, 2)).unwrap() / num_colors_per_channel;
            let quantized_pixel = num_colors_per_channel.pow(2) * r_pixel + num_colors_per_channel * g_pixel + b_pixel;
            quantized_img[[i, j]] = quantized_pixel;
        }
    }
    
    // Convert image to string grid
    let mut grid_str = String::new();
    for i in 0..quantized_img.dim().0 {
        for j in 0..quantized_img.dim().1 {
            grid_str.push_str(&quantized_img[[i, j]].to_string());
        }
        grid_str.push_str("\n");
    }

    Ok((original_data_url, grid_str))
}


#[component]
fn App() -> impl IntoView {
    let file_input: NodeRef<Input> = NodeRef::new();
    let (original_img_src, set_original_img_src) = signal(String::new());
    let (is_processing, set_is_processing) = signal(false);
    let (error_message, set_error_message) = signal(String::new());
    let (grid_str, set_grid_str) = signal(String::new());



    let update_image = move |_| {
        spawn_local(async move {
                let input_element = file_input.get();
                set_original_img_src.set(convert_image_input_to_base_64(input_element).await.unwrap_or_default());
                let input_element = file_input.get();
                let (original_data_url, grid_str) = process_image(input_element).await.unwrap_or_default();
                // set_original_img_src.set(original_data_url);
                set_grid_str.set(grid_str);
            }
        )
    };

    view! {
        <div class="container">
            <h1>"TGV Image Denoising"</h1>

            <div class="upload-section">
                <input 
                    type="file" 
                    accept="image/*" 
                    node_ref=file_input
                    // on:
                    on:change=update_image
                />
            </div>


            <div class="image-container">
                <Show when=move || !original_img_src.get().is_empty()>
                    <div class="image-box">
                        <h2>"Original Image"</h2>
                        <img src=original_img_src alt="Original Image" />
                    </div>
                </Show>

            </div>

            <div class="grid-container">
                <h2>"Grid"</h2>
                <pre>{grid_str}</pre>
            </div>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::attr::csp("worker-src 'self' blob:;");
    leptos::mount::mount_to_body(App)
}
