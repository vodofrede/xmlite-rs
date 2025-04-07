use std::fs;

#[test]
fn large_file() -> Result<(), &'static str> {
    let file = fs::read_to_string("tests/wayland.xml").unwrap();
    let xml = xml::parse(&file)?;
    eprintln!("{xml:?}");
    Ok(())
}
