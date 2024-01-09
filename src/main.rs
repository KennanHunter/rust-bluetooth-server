use std::borrow::Borrow;
use std::io::{Error, Write};

use async_std::{io, task};
use windows::core::HSTRING;
use windows::Devices::Bluetooth::Rfcomm::{RfcommServiceId, RfcommServiceProvider};
use windows::Foundation::TypedEventHandler;
use windows::Networking::Sockets::{
    SocketProtectionLevel, StreamSocketListener, StreamSocketListenerConnectionReceivedEventArgs,
};

fn main() -> Result<(), Error> {
    let create_server_task = task::spawn(async { create_server().await });

    task::block_on(create_server_task);

    Ok(())
}

async fn create_server() {
    // let available_devices = DeviceInformation::FindAllAsync().unwrap().await.unwrap();

    // println!("Available Devices found");

    // let device_id = &available_devices
    //     .First()
    //     .unwrap()
    //     .next()
    //     .unwrap()
    //     .Id()
    //     .unwrap();

    // println!("Device Id found");

    // let device = RfcommDeviceService::FromIdAsync(device_id).unwrap();

    let server_service_id = RfcommServiceId::ObexFileTransfer().unwrap();
    let server_provider = RfcommServiceProvider::CreateAsync(&server_service_id).unwrap();

    let listener = StreamSocketListener::new().unwrap();

    let handler: TypedEventHandler<
        StreamSocketListener,
        StreamSocketListenerConnectionReceivedEventArgs,
    > = TypedEventHandler::new(|_, _| -> windows::core::Result<()> {
        println!("New connection");

        Ok(())
    });
    // listener.ConnectionReceived(|| {});

    listener
        .BindServiceNameWithProtectionLevelAsync(
            &server_service_id.AsString().unwrap(),
            SocketProtectionLevel::BluetoothEncryptionAllowNullAuthentication,
        )
        .unwrap();

    listener.ConnectionReceived(&handler).unwrap();

    let mut input_sink = String::new();

    std::io::stdin()
        .read_line(&mut input_sink)
        .expect("Failed to read line");
}
