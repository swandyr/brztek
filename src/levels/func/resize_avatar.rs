use tracing::instrument;

// Change .webp extension to .png and remove parameters from URL
#[instrument]
pub fn resize_avatar(mut url: String) -> String {
    if let Some(index) = url.find("webp") {
        let _: String = url.split_off(index);
        url.push_str("png?size=96"); // Ensure the size of the image to be at max 96x96
    }
    url
}
