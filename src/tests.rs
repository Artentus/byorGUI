#[cfg(miri)]
#[test]
fn test_ub() {
    use crate::*;

    let mut gui = ByorGui::default();

    gui.frame(
        Vec2 {
            x: 800.px(),
            y: 600.px(),
        },
        1.0,
        MouseState::default(),
        |mut gui| {
            for _ in 0..100 {
                gui.insert_node(None, &Style::DEFAULT)?;
            }

            Result::<(), DuplicateUidError>::Ok(())
        },
    )
    .expect("error building GUI");
}
