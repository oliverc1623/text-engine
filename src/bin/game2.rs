use rand::{thread_rng, Rng};
use std::io;
use std::io::Write;
// use std::thread::sleep;
use std::{io::stdout, time::Duration};
use text_engine::*;

pub use crossterm::{
    cursor::position,
    event::{self, poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
    Command, Result,
};

fn generate_map(room: &text_engine::Room) -> Vec<Vec<char>> {
    let mut map = vec![vec!['.'; room.w]; room.h];
    for cell in map[0].iter_mut() {
        *cell = '#';
    }
    for cell in map[room.h - 1].iter_mut() {
        *cell = '#';
    }
    for row in map[1..(room.h - 1)].iter_mut() {
        row[0] = '#';
        row[room.w - 1] = '#';
    }
    map[1][0] = '|'; // initialize backdoor
    for i in 0..room.doors.len() {
        map[thread_rng().gen_range(1, room.h - 1)][room.w - 1] = '|';
    }
    // add items to map
    for item in room.items.iter() {
        let mut rand_y = thread_rng().gen_range(1, room.h - 1);
        let mut rand_x = thread_rng().gen_range(1, room.w - 1);
        // make sure item coordinates are not the same as player default position
        while rand_y == 1 && rand_x == 1 {
            rand_y = thread_rng().gen_range(1, room.h - 1);
            rand_x = thread_rng().gen_range(1, room.w - 1);
        }
        let item_char = match item {
            Item::Sword => '+',
            Item::Key => '?',
        };
        map[rand_y][rand_x] = item_char;
    }
    // add enemies to map
    for i in 0..room.enemies {
        let mut rand_y = thread_rng().gen_range(1, room.h - 1);
        let mut rand_x = thread_rng().gen_range(1, room.w - 1);
        // make sure item coordinates are not the same as player default position
        while rand_y == 1 && rand_x == 1 || map[rand_y][rand_x] != '.' {
            rand_y = thread_rng().gen_range(1, room.h - 1);
            rand_x = thread_rng().gen_range(1, room.w - 1);
        }
        map[rand_y][rand_x] = '*';
    }
    map
}

fn print_events() -> Result<()> {
    let rooms = [
        Room {
            name: "Main Room",
            desc: "This baroque style living room looks fancy. Let's see what the other doors lead too.",
            doors: vec![Door{target:RoomID(1), triggers:vec!["door", "north", "go north"], message:None, condition:Some(Item::Key)}],
            items: vec![Item::Sword, Item::Key],
            w: 7,
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
            items: vec![Item::Sword, Item::Key],
            w: 5,
            h: 8,
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

    let mut inventory: Vec<Item> = vec![];
    let end_rooms = [RoomID(2), RoomID(3)];
    let mut at = RoomID(0);
    let mut room_items: Vec<Vec<Item>> = vec![];
    let mut player_pos = (1, 1); // (y,x)
    let player_char = '@';
    let mut player_hp = 2;
    let mut player_score = 0;
    // let width: usize = thread_rng().gen_range(6, 10);
    // let height: usize = thread_rng().gen_range(6, 10);
    let mut map = generate_map(&rooms[at.0]);
    loop {
        let here = &rooms[at.0];
        let mut prev_point = map[player_pos.0][player_pos.1];
        map[player_pos.0][player_pos.1] = player_char;
        if player_hp == 0{
            break;
        }
        
        println!("Score: {}", player_score);
        println!("HP: {}", player_hp);
        println!("Player inventory: {:?}", inventory);
        for row in map.iter() {
            for c in row.iter() {
                print!("   {}", c);
            }
            println!();
        }
        // Wait up to 1s for another event
        if poll(Duration::from_millis(1_000))? {
            // It's guaranteed that read() wont block if `poll` returns `Ok(true)`
            let event = read()?;

            if event == Event::Key(KeyCode::Up.into()) {
                map[player_pos.0][player_pos.1] = prev_point;
                player_pos.0 -= 1;
            }
            if event == Event::Key(KeyCode::Down.into()) {
                map[player_pos.0][player_pos.1] = prev_point;
                player_pos.0 += 1;
            }
            if event == Event::Key(KeyCode::Left.into()) {
                map[player_pos.0][player_pos.1] = prev_point;
                player_pos.1 -= 1;
            }
            if event == Event::Key(KeyCode::Right.into()) {
                map[player_pos.0][player_pos.1] = prev_point;
                player_pos.1 += 1;
            }
            if event == Event::Key(KeyCode::Char('x').into()){
                if inventory.contains(&Item::Sword){
                    if map[player_pos.0][player_pos.1 + 1] == '*' {
                        map[player_pos.0][player_pos.1 + 1] = '.';
                    }
                    if map[player_pos.0 + 1][player_pos.1] == '*' {
                        map[player_pos.0 + 1][player_pos.1] = '.';
                    }
                    if map[player_pos.0 + 1][player_pos.1 + 1] == '*' {
                        map[player_pos.0 + 1][player_pos.1 + 1] = '.';
                    }
                    if map[player_pos.0 - 1][player_pos.1] == '*' {
                        map[player_pos.0 - 1][player_pos.1] = '.';
                    }
                    if map[player_pos.0][player_pos.1 - 1] == '*' {
                        map[player_pos.0][player_pos.1 - 1] = '.';
                    }
                    if map[player_pos.0 - 1][player_pos.1 - 1] == '*' {
                        map[player_pos.0 - 1][player_pos.1 - 1] = '.';
                    }
                    if map[player_pos.0 - 1][player_pos.1 + 1] == '*' {
                        map[player_pos.0 - 1][player_pos.1 + 1] = '.';
                    }
                    if map[player_pos.0 + 1][player_pos.1 - 1] == '*' {
                        map[player_pos.0 + 1][player_pos.1 - 1] = '.';
                    }
                }                
            }
            if event == Event::Key(KeyCode::Esc.into()) {
                break;
            }
        } else {
            // Timeout expired, no event for 1s
            let mut rand_y = thread_rng().gen_range(1, map[0].len() - 1);
            let mut rand_x = thread_rng().gen_range(1, map.len() - 1);
            // make sure item coordinates are not the same as player default position
            while rand_y == 1 && rand_x == 1 || map[rand_y][rand_x] != '.' {
                rand_y = thread_rng().gen_range(1, map[0].len() - 1);
                rand_x = thread_rng().gen_range(1, map.len() - 1);
            }
            map[rand_y][rand_x] = '*';
        }

        match map[player_pos.0][player_pos.1] {
            '+' => inventory.push(Item::Sword),
            '?' => inventory.push(Item::Key),
            '*' => {
                if !inventory.contains(&Item::Sword) {
                    player_hp -= 1;
                    player_score -= 100;
                } else {
                    player_score += 100;
                }
            }
            '#' => player_pos = (1, 1),
            '|' => {
                println!("at door");
                prev_point = '|';
            }
            _ => {prev_point = '.';}
        };
        // update last player position to be '.'
        map[player_pos.0][player_pos.1] = prev_point;
    }
    Ok(())
}

fn start_game() {
    let mut input = String::new();
    let mut pass = false;
    while !pass {
        io::stdout().flush().unwrap();
        input.clear();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "s" {
            println!("Good Luck!");
            pass = true;
        }
    }
}

fn main() -> Result<()> {
    io::stdout().flush().unwrap();
    println!("Plathorax: Desert Quest");
    println!("============================");
    println!(
        "You have been wondering the Great Desert for 
     days when suddenly to ride upon an oasis. 
     At the nearest pond, you hastily get off your horse 
     and submerge your head in the water, quenching your thirst.
     Once, you finish drinking you look across the pond and see 
     a house. As you approach the house to realize it is actually a mansion. 
     You enter the mansion and your quest begins."
    );
    println!("============================");
    println!(
        "RULES: 
    - use arrow keys to navigate your 
    - if an item is in your inventory simply walk to the object to use it
    - avoid catus"
    );

    Duration::from_millis(1_000);
    start_game();
    Duration::from_millis(1_000);

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnableMouseCapture)?;
    if let Err(e) = print_events() {
        println!("Error: {:?}\r", e);
    }
    println!("Game Over");
    execute!(stdout, DisableMouseCapture)?;
    disable_raw_mode()
}
