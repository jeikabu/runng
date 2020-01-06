use runng::mem::*;

#[test]
fn string() -> runng::Result<()> {
    let _unused = NngString::new("test")?;
    let _unused = NngString::new(vec![0]).expect_err("bytes with nul should fail");
    Ok(())
}

#[test]
fn alloc() -> runng::Result<()> {
    Ok(())
}
