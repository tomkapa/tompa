use server::openapi::ApiDoc;
use utoipa::OpenApi;

fn main() {
    print!("{}", ApiDoc::openapi().to_json().expect("OpenAPI serialization failed"));
}
