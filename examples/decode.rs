use std::fs::File;

use ogg::reading::PacketReader;
use opus_rs::{
    ogg::{OggOpusHead, OggOpusTags},
    opus::OpusPacket,
};

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
