use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    shadow_rs::new().map_err(|err| err.to_string())?;
    Ok(())
}
