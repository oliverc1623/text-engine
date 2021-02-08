use text_engine::*;

fn main() {
    use std::io;
    // We need the Write trait so we can flush stdout
    use std::io::Write;

    let rooms = [
        Room {
            name: "Foyer",
            desc: "This beautifully decorated foyer beckons you further into the mansion.  There is a door to the north.",
            doors: vec![Door{target:RoomID(1), triggers:vec!["door", "north", "go north"], message:None, condition:None}],
            items: vec![],
            w: 5,
            h: 7,
            enemies: 3
        },
        Room {
            name: "Antechamber",
            desc: "Dark wood paneling covers the walls.  An intricate painting of a field mouse hangs slightly askew on the wall (it looks like you could fix it).  The gilded northern doorway lies open to a shadowy parlour.  You can return to the foyer to the southern door. Items: Sword, Key, Bow, Arrow, and Bedroll",
            doors: vec![
                Door{target:RoomID(0), triggers:vec!["door", "south", "go south", "foyer"], message:None, condition:None},
                Door{target:RoomID(2), triggers:vec!["north", "doorway", "go north"], message:None, condition:Some(Item::Sword)},
                Door{target:RoomID(3), triggers:vec!["painting", "mouse", "fix painting"], message:Some("As you adjust the painting, a trap-door opens beneath your feet!"), 
                condition:Some(Item::Key)}
            ],
            items: vec![Item::Sword, Item::Key, Item::Bow, Item::Arrow, Item::Bedroll],
            w: 5,
            h: 7,
            enemies: 3
        },
        Room {
            name: "A Room Full of Snakes!",
            desc: "The shadows wriggle and shift as you enter the parlour.  The floor is covered in snakes!  The walls are covered in snakes!  The ceiling is covered in snakes!  You are also covered in snakes!\n\nBAD END",
            doors:vec![],
            items: vec![],
            w: 5,
            h: 7,
            enemies: 3
        },
        Room {
            name: "The Vault",
            desc: "When you regain consciousness, you feel a stabbing sensation in your lower back.  Reaching beneath you, you discover a massive diamond!  This room is full of gold and jewels, and a convenient ladder leading back outdoors!\n\nYou win!",
            doors:vec![],
            items:vec![],
            w: 5,
            h: 7,
            enemies: 3
        }
    ];

    let inventory: Vec<Item> = vec![];

    let end_rooms = [RoomID(2), RoomID(3)];
    let mut input = String::new();

    let mut at = RoomID(0);
    let mut room_items: Vec<Vec<Item>> = vec![];
    println!("The Spooky Mansion Adventure");
    println!("============================");
    println!();
    println!("You've been walking for hours in the countryside, and have finally stumbled on the spooky mansion you read about in the tour guide.");
    loop {
        // We don't want to move out of rooms, so we take a reference
        let here = &rooms[at.0];
        println!("{}\n{}", here.name, here.desc);
        room_items.push(here.items.clone());
        if end_rooms.contains(&at) {
            break;
        }
        loop {
            print!("What will you do?\n> ");
            println!("Your Items: {:?}", inventory);
            io::stdout().flush().unwrap();
            input.clear();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            if let Some(door) = here.doors.iter().find(|d| d.triggers.contains(&input)) {
                if let Some(msg) = door.message {
                    println!("{}", msg);
                }
                at = door.target;
                break;
            } else {
                println!("You can't do that!");
            }
        }
    }
}
