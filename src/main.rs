use bytes::BufMut;
use futures::TryStreamExt;
use warp::{
    http::StatusCode,
    multipart::{FormData, Part},
    Filter, Rejection, Reply,
};
use std::convert::Infallible;


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


            // let file_name = format!("./files/{}.{}","unknown", file_ending);
            // tokio::fs::write(&file_name, value).await.map_err(|e| {
            //     eprint!("error writing file: {}", e);
            //     warp::reject::reject()
            // })?;
            //TODO:
            // 1. Use cavif-rs code for jpg/png conversion
            // 2. Encode using default params
            // 3. Return image
            // 4. Implement params from URL

            return Ok(format!("Got {} bytes", value.len()))

        }
    }

    Ok("success".to_string())
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
    // GET /hello/warp => 200 OK with body "Hello, warp!"

    let upload_route = warp::path("upload")
        .and(warp::post())
        .and(warp::multipart::form().max_length(5_000_000))
        .and_then(upload);
    let download_route = warp::path("files").and(warp::fs::dir("./files/"));

    let router = upload_route.or(download_route).recover(handle_rejection);


    println!("Server started at http://127.0.0.1:3030");
    warp::serve(router).run(([127, 0, 0, 1], 3030)).await;
}