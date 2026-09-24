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

    assert_eq!(
        s.brief,
        "Adds targets to an admin menu.\n\nEach client is displayed as: name (userid)\nEach item contains the userid as a string for its info."
    );
    assert_eq!(s.tag("param:menu"), Some("Menu Handle."));
    assert_eq!(s.tag("param:source_client"), Some("Source client, or 0 to ignore immunity."));
    assert_eq!(s.tag("param:alive_only"), Some("True to only select alive players."));
    assert_eq!(s.tag("return"), Some("Number of clients added."));
    assert_eq!(s.tags.len(), 6);
}
