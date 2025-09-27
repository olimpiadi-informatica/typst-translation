pub fn embedded_font_files() -> impl Iterator<Item = &'static [u8]> {
    [
        include_bytes!("../../fonts/lmmono-italic.ttf") as &[_],
        include_bytes!("../../fonts/lmmono-regular.ttf"),
        include_bytes!("../../fonts/lmroman-bolditalic.ttf"),
        include_bytes!("../../fonts/lmroman-bold.ttf"),
        include_bytes!("../../fonts/lmroman-italic.ttf"),
        include_bytes!("../../fonts/lmroman-regular.ttf"),
        include_bytes!("../../fonts/majalla.ttf"),
        include_bytes!("../../fonts/majallab.ttf"),
    ]
    .into_iter()
}
