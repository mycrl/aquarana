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
        OggOpusHead::decode(&packet.data).unwrap()
    );

    let packet = reader.read_packet().unwrap().unwrap();
    println!(
        "opus tags = {:#?}",
        OggOpusTags::decode(&packet.data).unwrap()
    );

    let packet = reader.read_packet().unwrap().unwrap();
    println!(
        "opus packet = {:#?}",
        OpusPacket::decode(&packet.data).unwrap()
    );
}
