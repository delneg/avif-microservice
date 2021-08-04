use bytes::BufMut;
use futures::TryStreamExt;
use warp::{
    http::StatusCode,
    multipart::{FormData, Part},
    Filter, Rejection, Reply,
};
use imgref::ImgVec;
use std::convert::Infallible;
use ravif::{RGBA8, Config, ColorSpace};
use warp::reply::Response;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

fn load_rgba(mut data: &[u8], premultiplied_alpha: bool) -> Result<ImgVec<RGBA8>, BoxError> {
    use rgb::FromSlice;

    let mut img = if data.get(0..4) == Some(&[0x89, b'P', b'N', b'G']) {
        let img = lodepng::decode32(data)?;
        ImgVec::new(img.buffer, img.width, img.height)
    } else {
        let mut jecoder = jpeg_decoder::Decoder::new(&mut data);
        let pixels = jecoder.decode()?;
        let info = jecoder.info().ok_or("Error reading JPEG info")?;
        use jpeg_decoder::PixelFormat::*;
        let buf: Vec<_> = match info.pixel_format {
            L8 => {
                pixels.iter().copied().map(|g| RGBA8::new(g, g, g, 255)).collect()
            }
            RGB24 => {
                let rgb = pixels.as_rgb();
                rgb.iter().map(|p| p.alpha(255)).collect()
            }
            CMYK32 => return Err("CMYK JPEG is not supported. Please convert to PNG first".into()),
        };
        ImgVec::new(buf, info.width.into(), info.height.into())
    };
    if premultiplied_alpha {
        img.pixels_mut().for_each(|px| {
            px.r = (px.r as u16 * px.a as u16 / 255) as u8;
            px.g = (px.g as u16 * px.a as u16 / 255) as u8;
            px.b = (px.b as u16 * px.a as u16 / 255) as u8;
        });
    }
    Ok(img)
}

async fn upload(form: FormData) -> Result<impl Reply, Rejection> {
    let parts: Vec<Part> = form.try_collect().await.map_err(|e| {
        eprintln!("form error: {}", e);
        warp::reject::reject()
    })?;
    if parts.len() > 1 {
        eprintln!("Too many parts - {}", parts.len());
        return Err(warp::reject::reject());
    }
    for p in parts {
        if p.name() == "file" {
            // let original_filename = p.filename();
            let content_type = p.content_type();
            let file_ending;
            match content_type {
                Some(file_type) => match file_type {
                    "image/jpeg" => {
                        file_ending = "jpg";
                    }
                    "image/png" => {
                        file_ending = "png";
                    }
                    v => {
                        eprintln!("invalid file type found: {}", v);
                        return Err(warp::reject::reject());
                    }
                },
                None => {
                    eprintln!("file type could not be determined");
                    return Err(warp::reject::reject());
                }
            }

            let value = p
                .stream()
                .try_fold(Vec::new(), |mut vec, data| {
                    vec.put(data);
                    async move { Ok(vec) }
                })
                .await
                .map_err(|e| {
                    eprintln!("reading file error: {}", e);
                    warp::reject::reject()
                })?;


            let premultiplied_alpha = false;
            let mut img = load_rgba(value.as_slice(), premultiplied_alpha)
                .map_err(|e| {
                    eprintln!("Loading image error {}", e);
                    warp::reject::reject()
                })?;
            let quality = 80. as f32;
            let alpha_quality = ((quality + 100.)/2.).min(quality + quality/4. + 2.);
            let config = &Config { quality, speed: 4, alpha_quality, premultiplied_alpha, color_space: ColorSpace::RGB, threads: 0 };
            let (out_data, color_size, alpha_size) = ravif::encode_rgba(img.as_ref(), config).map_err(|e| {
                eprintln!("Encoding error {}", e);
                warp::reject::reject()
            })?;
            println!("Success: {}KB ({}B color, {}B alpha, {}B HEIF)", (out_data.len()+999)/1000, color_size, alpha_size, out_data.len() - color_size - alpha_size);


            return Ok(out_data);

        }
    }

    eprintln!("Didn't find any parts");
    return Err(warp::reject::reject());
}

async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (StatusCode::BAD_REQUEST, "Payload too large".to_string())
    } else {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    Ok(warp::reply::with_status(message, code))
}

#[tokio::main]
async fn main() {

    //TODO:
    // 1. Use cavif-rs code for jpg/png conversion +
    // 2. Encode using default params +
    // 3. Return image +
    // 4. Implement params from URL
    // 5. Test from html
    let upload_route = warp::path("upload")
        .and(warp::post())
        .and(warp::multipart::form().max_length(5_000_000))
        .and_then(upload);
    let download_route = warp::path("files").and(warp::fs::dir("./files/"));

    let router = upload_route.or(download_route).recover(handle_rejection);


    println!("Server started at http://127.0.0.1:3030");
    warp::serve(router).run(([127, 0, 0, 1], 3030)).await;
}