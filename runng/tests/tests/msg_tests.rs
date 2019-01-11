use runng::*;

#[test]
fn msg() -> NngReturn {
    let mut builder = msg::MsgBuilder::default();
    let value: u32 = 0x0123_4567;
    builder.append_u32(value);
    let mut msg = builder.build()?;
    assert_eq!(value, msg.trim_u32()?);

    let data = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let mut msg = builder.clean().append_slice(&data).build()?;
    let mut nngmsg = msg::NngMsg::create()?;
    nngmsg.append(data.as_ptr(), data.len())?;
    assert_eq!(nngmsg.body(), msg.body());

    Ok(())
}
