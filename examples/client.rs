use anyhow::Result;
use titan::Titan;

fn main() -> Result<()> {
    let mut titan = Titan::new()?;

    // engine.create_window
    // let viking_room = engine.load_obj("viking_room")

    // viking_room.instances = [
    //     pos1, pos2, pos3, pos4
    // ]
    titan.run("Vulkan in Rust")
}
