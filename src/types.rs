//! Type definitions for this app.

use crate::entity::ID;
use crate::world::World;

//------------------------------------------------------------------------------------------------
// Basic Types

/// Directions
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Dir {
    North,
    South,
    East,
    West,
    Up,
    Down,
    In,
    Out,
}

/// The different kinds of prose supported by an entity.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum ProseType {
    /// Prose describing a room's interior
    Room,

    /// Prose describing a thing's visible appearance.
    Thing,

    /// The prose contents of a book, note, etc.
    Book,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
#[allow(dead_code)]
/// Game flags.  At present this is a mixture of engine flags and scenario flags.
pub enum Flag {
    /// This rule should only be fired once.
    FireOnce,

    /// This rule has fired at least once.
    Fired,

    /// Has the entity been killed?
    Dead,

    /// Has this entity been seen by the player?  Used mostly for locations.
    Seen(ID),

    /// Is the thing Scenery?
    Scenery,

    /// Does the player have dirty hands?
    DirtyHands,

    /// Does the location have clean water?
    HasWater,

    /// Is the thing dirty?
    Dirty,
}

/// A closure to produce a string from an entity
pub type EntityStringHook = &'static Fn(&World, ID) -> String;

/// Actions taken by rules (and maybe other things)
/// TODO: Move this to types, and define ActionScript.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Action {
    /// Print the entity's visual
    Print(String),

    /// Set the variable for the entity with the given ID
    SetFlag(ID, Flag),

    /// Clear the given variable
    ClearFlag(ID, Flag),

    /// Swap an item in the world for one in LIMBO
    Swap(ID, ID),
}
