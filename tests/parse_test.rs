extern crate spdcp;

const RAW: &'static str = "/**
* Adds targets to an admin menu.
*
* Each client is displayed as: name (userid)
* Each item contains the userid as a string for its info.
*
* @param menu          Menu Handle.
* @param source_client Source client, or 0 to ignore immunity.
* @param in_game_only  True to only select in-game players.
* @param alive_only    True to only select alive players.
* @return              Number of clients added.
*/";

#[test]
fn parse_test() {
    let s = spdcp::Comment::parse(RAW);

    println!("{:?}", s);
}
