//! The Player Control System

use crate::debug;
use crate::types::Detail::*;
use crate::types::Dir::*;
use crate::types::Var::*;
use crate::types::*;
use crate::world::*;
use crate::entity::Room;
use crate::entity::Player;

/// An error result
type CmdResult = Result<(), String>;

/// The Player Control system.  Processes player commands.
pub fn system(world: &mut World, command: &str) {
    let player = &mut world.get(world.pid).as_player();

    let tokens: Vec<&str> = command.split_whitespace().collect();

    // TODO: Map synonyms, remove punctuation, before pattern matching

    let result = match tokens.as_slice() {
        ["n"] => cmd_go(world, player, North),
        ["north"] => cmd_go(world, player, North),
        ["s"] => cmd_go(world, player, South),
        ["south"] => cmd_go(world, player, South),
        ["e"] => cmd_go(world, player, East),
        ["east"] => cmd_go(world, player, East),
        ["w"] => cmd_go(world, player, West),
        ["west"] => cmd_go(world, player, West),
        ["help"] => cmd_help(),
        ["look"] => cmd_look(world, player),
        ["i"] => cmd_inventory(world, player),
        ["invent"] => cmd_inventory(world, player),
        ["inventory"] => cmd_inventory(world, player),
        ["x", "self"] => cmd_examine_self(world, player),
        ["x", "me"] => cmd_examine_self(world, player),
        ["x", name] => cmd_examine(world, name),
        ["examine", "self"] => cmd_examine_self(world, player),
        ["examine", "me"] => cmd_examine_self(world, player),
        ["examine", name] => cmd_examine(world, name),
        ["get", name] => cmd_get(world, name),
        ["drop", name] => cmd_drop(world, name),
        ["wash", "hands"] => cmd_wash_hands(world),
        ["wash", _] => Err("Whatever for?".into()),
        ["exit"] => cmd_quit(world),
        ["quit"] => cmd_quit(world),

        // Debugging
        ["dump", id_arg] => cmd_dump(world, id_arg),
        ["dump"] => cmd_dump_world(world),
        ["list"] => cmd_list(world),

        // Error
        _ => Err("I don't understand.".into()),
    };

    // NEXT, handle the result
    if let Err(msg) = result {
        println!("{}\n", msg);
    } else {
        player.save(world);
    }
}

// User Commands

/// Move the player in the given direction
fn cmd_go(world: &mut World, player: &mut Player, dir: Dir) -> CmdResult {
    if let Some(dest) = world.follow(player.loc, dir) {
        let room = &world.get(dest).as_room();
        player.loc = dest;

        let seen = world.is(&Seen(dest));  // TODO: should be player property

        if !seen {
            describe_location(world, room, Full);
        } else {
            describe_location(world, room, Brief);
        }

        world.set(Seen(dest));  // TODO: should be player property
        Ok(())
    } else {
        Err("You can't go that way.".into())
    }
}

/// Display basic help, i.e., what commands are available.
fn cmd_help() -> CmdResult {
    println!(
        "\
You've got the usual commands: n, s, e, w, look, get, drop, quit.
You know.  Like that.
    "
    );

    Ok(())
}

/// Re-describe the current location.
fn cmd_look(world: &World, player: &Player) -> CmdResult {
    let room = &world.get(player.loc).as_room();
    describe_location(world, room, Full);
    Ok(())
}

/// Re-describe the current location.
fn cmd_inventory(world: &World, player: &Player) -> CmdResult {
    if player.inventory.is_empty() {
        println!("You aren't carrying anything.\n");
    } else {
        println!("You have: {}.\n", invent_list(world, &player.inventory));
    }
    Ok(())
}

/// Describe a thing in the current location.
fn cmd_examine(world: &World, name: &str) -> CmdResult {
    if let Some(id) = find_visible_thing(world, name) {
        println!("{}\n", world.prose(id));
        Ok(())
    } else {
        Err("You don't see any such thing.".into())
    }
}

/// Describe a thing in the current location.
fn cmd_examine_self(world: &World, player: &Player) -> CmdResult {
    let mut msg = String::new();

    msg.push_str(&player.prose);

    if world.is(&DirtyHands) {
        msg.push_str(" Your hands are kind of dirty, though.");
    } else {
        msg.push_str(" Plus, they're clean bits!");
    }
    println!("{}\n", msg);

    Ok(())
}

fn cmd_wash_hands(world: &mut World) -> CmdResult {
    let loc = world.loc(world.pid);

    // TODO: Use room and property component
    if !world.is(&HasWater(loc)) {
        return Err("That'd be a neat trick.".into());
    }

    let mut msg = String::new();
    msg.push_str("You wash your hands in the water.");

    if world.is(&DirtyHands) {
        msg.push_str(" They look much cleaner.");
        world.clear(&DirtyHands);
    }

    println!("{}\n", msg);

    Ok(())
}

/// Gets a thing from the location's inventory.
fn cmd_get(world: &mut World, name: &str) -> CmdResult {
    let loc = here(world);
    if find_in_inventory(world, world.pid, name).is_some() {
        Err("You already have it.".into())
    } else if find_in_scenery(world, loc, name).is_some() {
        Err("You can't take that!".into())
    } else if let Some(id) = find_in_inventory(world, loc, name) {
        world.take_out(id, loc);
        world.put_in(id, world.pid);
        println!("Taken.\n");
        Ok(())
    } else {
        Err("You don't see any such thing.".into())
    }
}

/// Drops a thing you're carrying
fn cmd_drop(world: &mut World, name: &str) -> CmdResult {
    let loc = here(world);
    if let Some(id) = find_in_inventory(world, world.pid, name) {
        world.take_out(id, world.pid);
        world.put_in(id, loc);
        println!("Dropped.\n");
        Ok(())
    } else if find_visible_thing(world, name).is_some() {
        Err("You aren't carrying that.".into())
    } else {
        Err("You don't see any such thing.".into())
    }
}

/// Quit the game.
fn cmd_quit(_world: &World) -> CmdResult {
    println!("Bye, then.");
    ::std::process::exit(0);
}

// Debugging commands

/// Dump information about the given entity, provided the ID string is valid.
fn cmd_dump(world: &World, id_arg: &str) -> CmdResult {
    let id = parse_id(world, id_arg)?;
    debug::dump_entity(world, id);
    Ok(())
}

/// Dump information about all entities.
fn cmd_dump_world(world: &World) -> CmdResult {
    debug::dump_world(world);
    Ok(())
}

/// List all of the available entities.
fn cmd_list(world: &World) -> CmdResult {
    debug::list_world(world);
    Ok(())
}

//-------------------------------------------------------------------------
// Actions
//
// These functions are used to implement the above commands.

/// Describe the location.
pub fn describe_location(world: &World, room: &Room, detail: Detail) {
    // FIRST, display the room's description
    if detail == Full {
        println!("{}\n{}\n", room.name, room.prose);
    } else {
        println!("{}\n", room.name);
    }

    // NEXT, list any objects in the room's inventory.  (We don't list
    // scenary; presumably that's in the description.)
    if !room.inventory.is_empty() {
        println!("You see: {}.\n", invent_list(world, &room.inventory));
    }
}

//-------------------------------------------------------------------------
// Parsing Tools

/// Parse a token as an entity ID, return an error result on failure.
fn parse_id(world: &World, token: &str) -> Result<ID, String> {
    let id = match token.parse() {
        Ok(id) => id,
        Err(_) => {
            return Err(format!("Not an ID: {}", token));
        }
    };

    if id >= world.entities.len() {
        return Err(format!("Out of range: {}", token));
    }

    Ok(id)
}

/// Find a visible thing: something you're carrying, or that's here in this location.
fn find_visible_thing(world: &World, name: &str) -> Option<ID> {
    let loc = here(world);

    if let Some(id) = find_in_inventory(world, world.pid, name) {
        return Some(id);
    }

    if let Some(id) = find_in_inventory(world, loc, name) {
        return Some(id);
    }

    if let Some(id) = find_in_scenery(world, loc, name) {
        return Some(id);
    }

    None
}

fn find_in_inventory(world: &World, loc: ID, name: &str) -> Option<ID> {
    if let Some(inv) = &world.entities[loc].inventory {
        for id in inv {
            if world.name(*id) == name {
                return Some(*id);
            }
        }
    }

    None
}

fn find_in_scenery(world: &World, loc: ID, name: &str) -> Option<ID> {
    for id in 1..world.entities.len() {
        if world.is_scenery(id) && world.loc(id) == loc && world.name(id) == name {
            return Some(id);
        }
    }

    None
}

fn here(world: &World) -> ID {
    world.loc(world.pid)
}

//-------------------------------------------------------------------------
// Display Tools

/// List the names of the entities, separated by commas.
/// TODO: This could probably be done with map and some kind of join function.
/// However, it seems that "join" is available in the nightly.
fn invent_list(world: &World, inventory: &Inventory) -> String {
    let mut list = String::new();

    for id in inventory {
        if !list.is_empty() {
            list.push_str(", ");
        }
        list.push_str(world.name(*id));
    }

    list
}
