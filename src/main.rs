use core::panic;
use std::io::Error;

use async_std::task;
use windows::core::{GUID, HSTRING};
use windows::Devices::Bluetooth::Rfcomm::{RfcommServiceId, RfcommServiceProvider};
use windows::Foundation::TypedEventHandler;
use windows::Networking::Sockets::{
    SocketProtectionLevel, StreamSocketListener, StreamSocketListenerConnectionReceivedEventArgs,
};
use windows::Storage::Streams::{DataWriter, UnicodeEncoding};

fn main() -> Result<(), Error> {
    task::block_on(async { create_server().await });

    Ok(())
}

static SERVICE_VERSION_ATTRIBUTE_ID: u32 = 0x0300;

// The SDP Type of the Service Name SDP attribute.
// The first byte in the SDP Attribute encodes the SDP Attribute Type as follows :
//    -  the Attribute Type size in the least significant 3 bits,
//    -  the SDP Attribute Type value in the most significant 5 bits.
static SERVICE_VERSION_ATTRIBUTE_TYPE: u8 = (4 << 3) | 5;
static MINIMUM_SERVICE_VERSION: u32 = 200;

async fn create_server() {
    let server_service_id =
        RfcommServiceId::FromUuid(GUID::from_u128(0xFD4E2478E0AC9184AE4E7EB0BA0844FC)).unwrap();
    // RfcommServiceId::GenericFileTransfer().unwrap();

    let server_provider = match RfcommServiceProvider::CreateAsync(&server_service_id)
        .unwrap()
        .await
    {
        Ok(service_provider) => service_provider,
        Err(err) if err.code().0 == 0x800710DFu32 as i32 => {
            panic!("Bluetooth capability not enabled")
        }
        Err(err) => panic!("Error created: {:#?}", err),
    };

    println!(
        "Service Provider created with ID: {}",
        server_provider.ServiceId().unwrap().AsString().unwrap()
    );

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
        Err(err) => panic!("Listener failed to bind: {:#?}", err),
    };

    listener.ConnectionReceived(&connection_handler).unwrap();

    /*
     * We need to get the map of SDP attributes that will be transmitted
     * when we are attempting to pair with the clients
     */
    match server_provider.SdpRawAttributes() {
        Ok(map) => {
            // We create a buffer that can will be bound to a specific attribute
            let data_writer = DataWriter::new().unwrap();

            data_writer
                .WriteByte(SERVICE_VERSION_ATTRIBUTE_TYPE)
                .unwrap();

            data_writer.WriteUInt32(MINIMUM_SERVICE_VERSION).unwrap();

            data_writer
                .SetUnicodeEncoding(UnicodeEncoding::Utf8)
                .unwrap();

            data_writer
                .WriteString(&HSTRING::from("Bluetooth Rfcomm Chat Service"))
                .unwrap();

            map.Insert(
                SERVICE_VERSION_ATTRIBUTE_ID,
                &data_writer.DetachBuffer().unwrap(),
            )
            .unwrap();
        }
        Err(err) => panic!("Error accessing SDP Raw Attributes: {:#?}", err),
    }

    println!("{:#?}", server_provider.SdpRawAttributes().unwrap());

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
