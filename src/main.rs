use core::panic;
use std::io::Error;

use async_std::task;
use windows::Devices::Bluetooth::Rfcomm::{RfcommServiceId, RfcommServiceProvider};
use windows::Foundation::TypedEventHandler;
use windows::Networking::Sockets::{
    SocketProtectionLevel, StreamSocketListener, StreamSocketListenerConnectionReceivedEventArgs,
};

fn main() -> Result<(), Error> {
    task::block_on(async { create_server().await });

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

    let server_provider = match RfcommServiceProvider::CreateAsync(&server_service_id)
        .unwrap()
        .await
    {
        Ok(service_provider) => {
            println!(
                "Service Provider created with ID: {}",
                service_provider.ServiceId().unwrap().AsString().unwrap()
            );
            service_provider
        }
        Err(err) => panic!("Error created: {:#?}", err),
    };

    let connection_handler: TypedEventHandler<
        StreamSocketListener,
        StreamSocketListenerConnectionReceivedEventArgs,
    > = TypedEventHandler::new(|_, args| -> windows::core::Result<()> {
        println!("New connection");
        println!("{:#?}", args);

        Ok(())
    });
    // listener.ConnectionReceived(|| {});

    let listener = match StreamSocketListener::new() {
        Ok(listener) => {
            println!("Socket Listener Object Created");
            listener
        }
        Err(err) => panic!("Socket listener failed to create: {:#?}", err),
    };

    match listener
        .BindServiceNameWithProtectionLevelAsync(
            &server_provider.ServiceId().unwrap().AsString().unwrap(),
            SocketProtectionLevel::BluetoothEncryptionAllowNullAuthentication,
        )
        .unwrap()
        .await
    {
        Ok(_) => println!("Listener binding succeeded"),
        Err(err) => panic!("Listener failed to bind {:#?}", err),
    };

    listener.ConnectionReceived(&connection_handler).unwrap();

    match server_provider.StartAdvertising(&listener) {
        Ok(_) => println!(
            "Server Advertising with ID {:?}",
            server_provider.ServiceId().unwrap().AsShortId().unwrap()
        ),
        Err(err) => panic!("Server could not start advertising: \n {:#?}", err),
    };

    let mut input_sink = String::new();

    std::io::stdin()
        .read_line(&mut input_sink)
        .expect("Failed to read line");
}
