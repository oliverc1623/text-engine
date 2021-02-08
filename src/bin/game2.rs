use rand::{thread_rng, Rng};
use std::io;
use std::io::Write;
use std::string::String;
use std::{io::stdout, time::Duration};
use text_engine::*;

pub use crossterm::{
    cursor,
    cursor::position,
    event::{self, poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute, queue, style,
    style::Color,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    Command, Result,
};

pub fn read_line() -> Result<String> {
    let mut line = String::new();
    while let Event::Key(KeyEvent { code, .. }) = event::read()? {
        match code {
            KeyCode::Enter => {
                break;
            }
            KeyCode::Char(c) => {
                line.push(c);
            }
            _ => {}
        }
    }

    Ok(line)
}

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
    for _i in 0..room.doors.len() {
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
            _ => '&',
        };
        map[rand_y][rand_x] = item_char;
    }
    // add enemies to map
    for _i in 0..room.enemies {
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

fn transition(here: &text_engine::Room) -> RoomID {
    let mut next_room_id: RoomID = RoomID(1);
    loop {
        io::stdout().flush().unwrap();
        println!("A door unlocks!");
        println!("{}", here.desc);
        let mut s: String;
        loop {
            print!("What will you do?\n> ");
            let input = read_line();
            s = input.unwrap();
            println!("You typed: {}", s);
            println!("Confirm? Type a number: 1) Yes 2) No");
            let confirmation = read_line();
            if confirmation.unwrap() == "1" {
                break;
            } 
        }        

        if let Some(door) = here.doors.iter().find(|d| {
            let words: Vec<&str> = s.as_str().split(' ').collect();
            let mut val: bool = false;
            for w in words.iter() {
                if d.triggers.contains(w) {
                    val = true;
                }
            }
            val
        }) {
            if let Some(msg) = door.message {
                println!("{}", msg);
            }
            next_room_id = door.target;
            break;
        } else {
            println!("You can't do that!");
        }
    }
    next_room_id
}

fn print_events<W: std::io::Write>(_w: &mut W) -> Result<()> {
    let rooms = [
        Room {
            name: "Main Room",
            desc: "This baroque style living room looks fancy. I see a door in the north side. I wonder where it will lead me?",
            doors: vec![Door{target:RoomID(1), triggers:vec!["door", "north", "go north"], message:None, condition:Some(Item::Key)}],
            items: vec![Item::Sword, Item::Key],
            w: 7,
            h: 7,
            enemies: 3
        },
        Room {
            name: "Living room",
            desc: "You make it to the living room. It is a beautifully decorated room except you're still not free from the cati. 
            in this room there are three doors. ",
            doors: vec![
                Door{target:RoomID(0), triggers:vec!["door", "south", "go south", "main"], message:None, condition:None},
                Door{target:RoomID(2), triggers:vec!["north", "doorway", "go north"], message:None, condition:Some(Item::Sword)},
                Door{target:RoomID(3), triggers:vec!["painting", "mouse", "fix painting"], message:Some("As you adjust the painting, a trap-door opens beneath your feet!"), 
                condition:Some(Item::Key)}
            ],
            items: vec![Item::Sword, Item::Key, Item::Bedroll],
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
    // let mut room_items: Vec<Vec<Item>> = vec![];
    let mut player_pos = (1, 1); // (y,x)
    let player_char = '@';
    let mut player_hp = 2;
    let mut player_score = 0;
    let mut here = &rooms[at.0];
    println!("{}", here.desc);
    println!("press enter to continue");
    println!("{:?}", read_line());
    // let width: usize = thread_rng().gen_range(6, 10);
    // let height: usize = thread_rng().gen_range(6, 10);
    let mut map = generate_map(&rooms[at.0]);
    loop {
        let mut prev_point = map[player_pos.0][player_pos.1];
        map[player_pos.0][player_pos.1] = player_char;
        if player_hp == 0 {
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
            if event == Event::Key(KeyCode::Char('x').into()) {
                if inventory.contains(&Item::Sword) {
                    if map[player_pos.0][player_pos.1 + 1] == '*' {
                        map[player_pos.0][player_pos.1 + 1] = '.';
                        player_score += 100;
                    }
                    if map[player_pos.0 + 1][player_pos.1] == '*' {
                        map[player_pos.0 + 1][player_pos.1] = '.';
                        player_score += 100;
                    }
                    if map[player_pos.0 + 1][player_pos.1 + 1] == '*' {
                        map[player_pos.0 + 1][player_pos.1 + 1] = '.';
                        player_score += 100;
                    }
                    if map[player_pos.0 - 1][player_pos.1] == '*' {
                        map[player_pos.0 - 1][player_pos.1] = '.';
                        player_score += 100;
                    }
                    if map[player_pos.0][player_pos.1 - 1] == '*' {
                        map[player_pos.0][player_pos.1 - 1] = '.';
                        player_score += 100;
                    }
                    if map[player_pos.0 - 1][player_pos.1 - 1] == '*' {
                        map[player_pos.0 - 1][player_pos.1 - 1] = '.';
                        player_score += 100;
                    }
                    if map[player_pos.0 - 1][player_pos.1 + 1] == '*' {
                        map[player_pos.0 - 1][player_pos.1 + 1] = '.';
                        player_score += 100;
                    }
                    if map[player_pos.0 + 1][player_pos.1 - 1] == '*' {
                        map[player_pos.0 + 1][player_pos.1 - 1] = '.';
                        player_score += 100;
                    }
                }
            }
            if event == Event::Key(KeyCode::Esc.into()) {
                break;
            }
        } else {
            // Timeout expired, no event for 1s
            let mut rand_x = thread_rng().gen_range(1, map[0].len() - 1);
            let mut rand_y = thread_rng().gen_range(1, map.len() - 1);
            // make sure item coordinates are not the same as player default position
            while (rand_y == 1 && rand_x == 1) || map[rand_y][rand_x] != '.' {
                rand_x = thread_rng().gen_range(1, map[0].len() - 1);
                rand_y = thread_rng().gen_range(1, map.len() - 1);
            }
            map[rand_y][rand_x] = '*';
        }

        match map[player_pos.0][player_pos.1] {
            '+' => {
                if !inventory.contains(&Item::Sword){
                    inventory.push(Item::Sword);
                }
            }
            '?' => {
                if !inventory.contains(&Item::Key){
                    inventory.push(Item::Key);
                }
            }
            '&' => {
                if !inventory.contains(&Item::Bedroll){
                    inventory.push(Item::Bedroll);
                }
            }
            '*' => {
                player_hp -= 1;
                player_score -= 100;
            }
            '#' => player_pos = (1, 1),
            '|' => {
                println!("at door");
                if inventory.contains(&Item::Key) {
                    at = transition(here);
                    if end_rooms.contains(&at) {
                        break;
                    }
                    map = generate_map(&rooms[at.0]);
                    here = &rooms[at.0];
                    player_pos = (1, 1);
                } else {
                    player_pos = (1, 1);
                }
            }
            _ => {
                prev_point = '.';
            }
        };
        map[player_pos.0][player_pos.1] = prev_point;
    }
    Ok(())
}

fn start_game() {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let name = input.trim();
    println!("Good Luck {}!", name);
}

fn main() -> Result<()> {
    io::stdout().flush().unwrap();
    println!("Cacti Cutter");
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
    println!("Type in your name and hit enter to get started!");

    Duration::from_millis(1_000);
    start_game();
    Duration::from_millis(1_000);

    enable_raw_mode()?;
    let mut stdout = stdout();
    // execute!(stdout, EnableMouseCapture)?;
    if let Err(e) = print_events(&mut stdout) {
        println!("Error: {:?}\r", e);
    }
    println!("Game Over");
    // execute!(stdout, DisableMouseCapture)?;
    disable_raw_mode()
}
