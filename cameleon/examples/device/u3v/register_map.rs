extern crate cameleon;
extern crate cameleon_device;

use cameleon::device::u3v::*;

fn main() {
    // Build emulator in case libusb is not supported.
    #[cfg(not(feature = "libusb"))]
    cameleon_device::u3v::EmulatorBuilder::new()
        .user_defined_name("cameleon-emulator")
        .unwrap()
        .build();

    // Enumerate devices.
    let mut devices = enumerate_devices().unwrap();
    if devices.is_empty() {
        println!("no device found");
        return;
    }

    // Open the first device.
    let mut device = devices.pop().unwrap();
    device.open().unwrap();

    //  Read ABRM.
    println!("\n### Technology Agnostic Boot Register Map ###\n");
    let abrm = device.abrm().unwrap();

    println!("gencp_version: {}", abrm.gencp_version().unwrap());
    println!("manufacturer_name: {}", abrm.manufacturer_name().unwrap());
    println!("model_name: {}", abrm.model_name().unwrap());
    println!("family_name: {:?}", abrm.family_name());
    println!("device_version: {}", abrm.device_version().unwrap());
    println!("manufacturer_name: {}", abrm.manufacturer_name().unwrap());
    println!("serial_number: {}", abrm.serial_number().unwrap());
    println!(
        "manifest_table_address: {}",
        abrm.manifest_table_address().unwrap()
    );
    println!("sbrm_address: {}", abrm.sbrm_address().unwrap());
    println!(
        "device_software_interface_version: {:?}",
        abrm.device_software_interface_version()
    );
    println!(
        "maximum_device_response_time: {:?}",
        abrm.maximum_device_response_time()
    );

    let device_capability = abrm.device_capability().unwrap();
    println!(
        "is_user_defined_name_supported: {}",
        device_capability.is_user_defined_name_supported()
    );
    println!("user_defined_name: {:?}", abrm.user_defined_name().unwrap());
    println!(
        "is_multi_event_supported: {}",
        device_capability.is_multi_event_supported()
    );
    println!(
        "is_multi_event_enabled: {}",
        abrm.device_configuration()
            .unwrap()
            .is_multi_event_enabled()
    );
    println!(
        "is_stacked_commands_supported: {}",
        device_capability.is_stacked_commands_supported()
    );

    // Write to registers.
    // NOTE. These oeprations will cause non-volatile changes to the register.
    //
    // abrm.set_user_defined_name("Cameleon").unwrap();
    // println!(
    //     "changed user_defined_name: {:?}",
    //     abrm.user_defined_name().unwrap()
    // );

    // if abrm.is_multi_event_supported() {
    //     abrm.enable_multi_event().unwrap();
    // }
    //

    //  Read SBRM.
    println!("\n### Technology Specifig Boot Register Map ###\n");

    let sbrm = abrm.sbrm().unwrap();
    println!("u3v_version: {}", sbrm.u3v_version().unwrap());
    println!(
        "maximum_command_transfer_length: {}",
        sbrm.maximum_command_transfer_length().unwrap()
    );
    println!(
        "maximum_acknowledge_transfer_length: {}",
        sbrm.maximum_acknowledge_trasfer_length().unwrap()
    );
    println!(
        "number_of_stream_channel: {}",
        sbrm.number_of_stream_channel().unwrap()
    );
    println!("sirm_address: {:?}", sbrm.sirm_address().unwrap());
    println!("sirm_length: {:?}", sbrm.sirm_length().unwrap());

    println!("eirm_address: {:?}", sbrm.eirm_address().unwrap());
    println!("eirm_length: {:?}", sbrm.eirm_length().unwrap());
    println!("iidc2_address: {:?}", sbrm.iidc2_address().unwrap());
    println!("current_speed: {:?}", sbrm.current_speed().unwrap());

    // Read manifest entries.
    let manifest_table = abrm.manifest_table().unwrap();
    for (i, entry) in manifest_table.entries().unwrap().enumerate() {
        println!("\n### Manifest Entry {}", i);
        println!(
            "GenICam file version: {}",
            entry.genicam_file_version().unwrap()
        );
        println!("GenICam file address: {}", entry.file_address().unwrap());
        println!("GenICam file size: {}", entry.file_size().unwrap());

        let file_info = entry.file_info().unwrap();
        println!(
            "GenICam file compression type: {:?}",
            file_info.compression_type().unwrap()
        );
        println!("GenICam file type: {:?}", file_info.file_type().unwrap());
        println!(
            "GenICam file schema version: {}",
            file_info.schema_version()
        );
    }
}
