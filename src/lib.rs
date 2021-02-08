#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub struct Room {
    pub name: &'static str, // E.g. "Antechamber"
    pub desc: &'static str, // E.g. "Dark wood paneling covers the walls.  The gilded northern doorway lies open."
    pub doors: Vec<Door>,
    pub items: Vec<Item>,
    pub w: usize,
    pub h: usize,
    pub enemies: usize,
}
pub struct Door {
    pub target: RoomID,                // More about this in a minute
    pub triggers: Vec<&'static str>,   // e.g. "go north", "north"
    pub message: Option<&'static str>, // What message, if any, to print when the doorway is traversed
    // Any other info about the door would go here
    pub condition: Option<Item>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct RoomID(pub usize);

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Item {
    Sword,
    Key,
    Bow,
    Arrow,
    Bedroll,
}

// enum GameMode {
//     Playing,
//     InventoryMenu,
//     ShopMenu
// }
