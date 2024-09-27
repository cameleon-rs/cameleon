use cameleon::gige::enumerate_cameras;

fn main() {
    let mut cameras = enumerate_cameras().unwrap();
    if cameras.is_empty() {
        println!("no camera found!");
        return;
    }

    let mut camera = cameras.pop().unwrap();
    camera.open().unwrap();
    camera.load_context().unwrap();

    camera.close().unwrap();
}
