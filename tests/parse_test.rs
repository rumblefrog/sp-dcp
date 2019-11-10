extern crate spdcp;

const RAW: &'static str = "/**
 * Gets a function id from a function name.
 *
 * @param plugin        Handle of the plugin that contains the function.
 *                      Pass INVALID_HANDLE to search in the calling plugin.
 * @param name          Name of the function.
 * @return              Function id or INVALID_FUNCTION if not found.
 * @error               Invalid or corrupt plugin handle.
 */";

#[test]
fn parse_test() {
    let s = spdcp::Comment::parse(RAW);

    println!("{:?}", s);
}
