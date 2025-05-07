use std::fs::File;

use aquarana::{OggOpusHead, OggOpusTags, opus::OpusPacket};
use ogg::reading::PacketReader;

fn main() {
    let file = File::open("./demo.opus").unwrap();

    let mut reader = PacketReader::new(file);

    let packet = reader.read_packet().unwrap().unwrap();
    println!(
        "opus head = {:#?}",
        OggOpusHead::try_from(packet.data.as_slice()).unwrap()
    );

    let packet = reader.read_packet().unwrap().unwrap();
    println!(
        "opus tags = {:#?}",
        OggOpusTags::try_from(packet.data.as_slice()).unwrap()
    );

    let packet = reader.read_packet().unwrap().unwrap();
    println!(
        "opus packet = {:#?}",
        OpusPacket::decode(packet.data.as_slice()).unwrap()
    );
}
