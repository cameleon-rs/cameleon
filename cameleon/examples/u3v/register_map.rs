/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cameleon::u3v::enumerate_cameras;

fn main() {
    // Enumerates cameras connected to the host.
    let mut cameras = enumerate_cameras().unwrap();

    if cameras.is_empty() {
        println!("no camera found!");
        return;
    }

    let mut camera = cameras.pop().unwrap();

    // Open the camera.
    camera.open().unwrap();

    let ctrl = &mut camera.ctrl;
    //  Read ABRM.
    println!("\n### Technology Agnostic Boot Register Map ###\n");
    let abrm = ctrl.abrm().unwrap();

    println!("gencp_version: {}", abrm.gencp_version(ctrl).unwrap());
    println!(
        "manufacturer_name: {}",
        abrm.manufacturer_name(ctrl).unwrap()
    );
    println!("model_name: {}", abrm.model_name(ctrl).unwrap());
    println!("family_name: {:?}", abrm.family_name(ctrl));
    println!("device_version: {}", abrm.device_version(ctrl).unwrap());
    println!(
        "manufacturer_name: {}",
        abrm.manufacturer_name(ctrl).unwrap()
    );
    println!("serial_number: {}", abrm.serial_number(ctrl).unwrap());
    println!(
        "manifest_table_address: {}",
        abrm.manifest_table_address(ctrl).unwrap()
    );
    println!("sbrm_address: {}", abrm.sbrm_address(ctrl).unwrap());
    println!(
        "device_software_interface_version: {:?}",
        abrm.device_software_interface_version(ctrl)
    );
    println!(
        "maximum_device_response_time: {:?}",
        abrm.maximum_device_response_time(ctrl)
    );

    let device_capability = abrm.device_capability().unwrap();
    println!(
        "is_user_defined_name_supported: {}",
        device_capability.is_user_defined_name_supported()
    );
    println!(
        "user_defined_name: {:?}",
        abrm.user_defined_name(ctrl).unwrap()
    );
    println!(
        "is_multi_event_supported: {}",
        device_capability.is_multi_event_supported()
    );
    println!(
        "is_multi_event_enabled: {}",
        abrm.device_configuration(ctrl)
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
    //     abrm.user_defined_name(ctrl).unwrap()
    // );

    // if abrm.is_multi_event_supported() {
    //     abrm.enable_multi_event(ctlr).unwrap();
    // }
    //

    //  Read SBRM.
    println!("\n### Technology Specifig Boot Register Map ###\n");

    let sbrm = ctrl.sbrm().unwrap();
    println!("u3v_version: {}", sbrm.u3v_version(ctrl).unwrap());
    println!(
        "maximum_command_transfer_length: {}",
        sbrm.maximum_command_transfer_length(ctrl).unwrap()
    );
    println!(
        "maximum_acknowledge_transfer_length: {}",
        sbrm.maximum_acknowledge_trasfer_length(ctrl).unwrap()
    );
    println!(
        "number_of_stream_channel: {}",
        sbrm.number_of_stream_channel(ctrl).unwrap()
    );
    println!("sirm_address: {:?}", sbrm.sirm_address(ctrl).unwrap());
    println!("sirm_length: {:?}", sbrm.sirm_length(ctrl).unwrap());

    println!("eirm_address: {:?}", sbrm.eirm_address(ctrl).unwrap());
    println!("eirm_length: {:?}", sbrm.eirm_length(ctrl).unwrap());
    println!("iidc2_address: {:?}", sbrm.iidc2_address(ctrl).unwrap());
    println!("current_speed: {:?}", sbrm.current_speed(ctrl).unwrap());

    // Read manifest entries.
    let manifest_table = abrm.manifest_table(ctrl).unwrap();
    for (i, entry) in manifest_table.entries(ctrl).unwrap().enumerate() {
        println!("\n### Manifest Entry {} ###\n", i);
        println!(
            "GenICam file version: {}",
            entry.genicam_file_version(ctrl).unwrap()
        );
        println!(
            "GenICam file address: {}",
            entry.file_address(ctrl).unwrap()
        );
        println!("GenICam file size: {}", entry.file_size(ctrl).unwrap());

        let file_info = entry.file_info(ctrl).unwrap();
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

    let sirm = ctrl.sirm().unwrap();

    println!("\n### Streaming Interface Register Map ###\n");
    println!(
        "payload_size_alignment: {}",
        sirm.payload_size_alignment(ctrl).unwrap()
    );

    println!("is_stream_enable: {}", sirm.is_stream_enable(ctrl).unwrap());
    println!(
        "required_payload_size: {}",
        sirm.required_payload_size(ctrl).unwrap()
    );
    println!(
        "required_leader_size: {}",
        sirm.required_leader_size(ctrl).unwrap()
    );
    println!(
        "required_trailer_size: {}",
        sirm.required_trailer_size(ctrl).unwrap()
    );
    println!(
        "maximum_leader_size: {}",
        sirm.maximum_leader_size(ctrl).unwrap()
    );
    println!(
        "maximum_trailer_size: {}",
        sirm.maximum_trailer_size(ctrl).unwrap()
    );
    println!(
        "payload_transfer_size: {}",
        sirm.payload_transfer_size(ctrl).unwrap()
    );
    println!(
        "payload_transfer_count: {}",
        sirm.payload_transfer_count(ctrl).unwrap()
    );
    println!(
        "payload_final_transfer1_size: {}",
        sirm.payload_final_transfer1_size(ctrl).unwrap()
    );
    println!(
        "payload_final_transfer2_size: {}",
        sirm.payload_final_transfer2_size(ctrl).unwrap()
    );
}
