use std::env;
use std::path::Path;
use testcontainers::core::WaitFor;
use testcontainers::images::generic::GenericImage;
use testcontainers::RunnableImage;

pub fn simple_opa_server() -> RunnableImage<GenericImage> {
    let image = GenericImage::new("openpolicyagent/opa", "latest")
        .with_exposed_port(8181)
        .with_wait_for(WaitFor::message_on_stderr("Server initialized"));

    let args = vec![
        "run".into(),
        "--server".into(),
        "--log-level=debug".into(),
        "/example".into(),
    ];

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let example = Path::new(&manifest_dir).join("example");

    RunnableImage::from((image, args))
        .with_volume((
            example.to_str().unwrap(),
            //"/Users/bob/repos/seedwing-io/opa-client/example/",
            "/example:Z",
        ))
        .with_mapped_port((8181, 8181))

    //RunnableImage::from((image, args)).with_mapped_port((8181, 8181))
}
