/// This example connects to a radio via serial, and demonstrates how to
/// configure handlers for different types of decoded radio packets.
/// https://meshtastic.org/docs/supported-hardware
extern crate meshtastic;

use std::env;
use std::io::{self, BufRead};

use meshtastic::api::StreamApi;
use meshtastic::utils;

// This import allows for decoding of mesh packets
// Re-export of prost::Message
use meshtastic::packet::PacketDestination;
use meshtastic::packet::PacketRouter;

// Assuming protobufs and NodeId are defined elsewhere
use meshtastic::protobufs::{FromRadio, MeshPacket};
//use meshtastic::connections::NodeId;
use meshtastic::types::NodeId;

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

// A simple error type
#[derive(Debug)]
struct TestRouterError(String);

impl Display for TestRouterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl Error for TestRouterError {}

// Metadata type for demonstration
struct HandlerMetadata {
    should_update_db: bool,
}

// Your packet router implementation
struct TestPacketRouter {
    my_id: NodeId,
}

impl PacketRouter<HandlerMetadata, TestRouterError> for TestPacketRouter {
    fn handle_packet_from_radio(
        &mut self,
        packet: FromRadio,
    ) -> Result<HandlerMetadata, TestRouterError> {
        // Check the packet
         println!("{:#?}", packet);

        Ok(HandlerMetadata {
            should_update_db: false,
        })
    }

    fn handle_mesh_packet(&mut self, packet: MeshPacket) -> Result<HandlerMetadata, TestRouterError> {
        // Check the packet
        println!("{:#?}", packet);

        Ok(HandlerMetadata {
            should_update_db: false,
        })
    }

    fn source_node_id(&self) -> NodeId {
        // Return the current node's ID
        println!("My_id requested: value is {}", self.my_id);
        self.my_id
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let (my_id, dest_id, message): (u32, u32, String);

    let args: Vec<_> = env::args().collect();

    if args.len() < 4 {
        panic!("Enter (space separated) your id, the destination id, a message:");        
    } else {
        // Parse source and destination
        my_id = args[1].parse().expect("Could not parse your id");
        dest_id = args[2].parse().expect("Could not parse destination id");
        // Remaining part as a string
        message = args[3].clone();
    }
    
    let stream_api = StreamApi::new();
    
    let available_ports = utils::stream::available_serial_ports()?;
    println!("Available ports: {:?}", available_ports);
    
    let entered_port = match available_ports.len() {
        1 => available_ports[0].clone(),
        _ => {
                println!("Enter the name of a port to connect to:");
                let stdin = io::stdin();
                stdin
                .lock()
                .lines()
                .next()
                .expect("Failed to find next line")
                .expect("Could not read next line").clone()
            }
        };
    
    let serial_stream = utils::stream::build_serial_stream(entered_port, None, None, None)?;
    let (mut _decoded_listener, stream_api) = stream_api.connect(serial_stream).await;
    
    let config_id = utils::generate_rand_id();
    let mut stream_api = stream_api.configure(config_id).await?;

    //let stream_api = StreamApi::new();
    //
    //println!("Enter the MAC address or name of a BLE device to connect to:");
    //
    //let stdin = io::stdin();
    //let entered_address = stdin
    //    .lock()
    //    .lines()
    //    .next()
    //    .expect("Failed to find next line")
    //    .expect("Could not read next line");
    //
    //let ble_stream = utils::stream::build_ble_stream(entered_address).await?;
    //let (mut decoded_listener, stream_api) = stream_api.connect(ble_stream).await;
    //
    //let config_id = utils::generate_rand_id();
    //let mut stream_api = stream_api.configure(config_id).await?;



    let mut router = TestPacketRouter { my_id: my_id.into() };
    stream_api.send_text(
            &mut router,
            message,
            PacketDestination::Node(dest_id.into()),
            true,
            0.into(),
        )
        .await?;


    let _stream_api = stream_api.disconnect().await?;

    Ok(())
}
