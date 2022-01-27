extern crate prost_build;

use std::{fs};
use std::path::Path;
// use protoc_rust::Customize;

use protobuf_codegen_pure::Codegen;
use protobuf_codegen_pure::Customize;

use std::io::Result;

fn main() -> Result<()>{
    let message_output = format!("/Users/r_kessler/Documents/uni/vorlesungen/masterprojekt/rust/src/messages/outs");

    if Path::new(&message_output).exists() {
        fs::remove_dir_all(&message_output).unwrap();
    }

    fs::create_dir(&message_output).unwrap();

    Codegen::new()
        .customize(Customize {
            gen_mod_rs: Some(true),
            ..Default::default()
        })
        .out_dir(message_output)
        .inputs(&["src/messages/controller_messages.proto", "src/messages/inter_agent_message.proto",
            "src/messages/stats_message.proto", "src/messages/authority_messages.proto"])
        .include("src/messages")
        .run()
        .expect("protoc");
    Ok(())
}