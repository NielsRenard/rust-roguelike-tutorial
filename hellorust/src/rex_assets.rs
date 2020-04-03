use rltk::rex::XpFile;

rltk::embedded_resource!(CAVE_ENTRANCE, "../resources/cave_entrance.xp");

pub struct RexAssets {
    pub menu: XpFile,
}

impl RexAssets {
    pub fn new() -> RexAssets {
        rltk::link_resource!(CAVE_ENTRANCE, "../resources/cave_entrance.xp");

        RexAssets {
            menu: XpFile::from_resource("../resources/cave_entrance.xp").unwrap(),
        }
    }
}
