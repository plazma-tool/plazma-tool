use plasma::dmo::Dmo;

#[test]
fn default_dmo() {
    let dmo = Dmo::default();

    // just testing for debug output really.
    //println!("{:#?}", dmo);

    assert_eq!(dmo.settings.start_full_screen, false);
}
