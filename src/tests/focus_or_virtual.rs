//! Tests for `window_virtual_north`/`window_virtual_south` acting as "focus
//! within the current stack if possible, otherwise switch virtual
//! workspace" — see `switch_virtual_workspace_bind` in
//! `src/ecs/workspace.rs`.

use bevy::prelude::*;

use crate::assert_focused;
use crate::commands::{Command, Direction, Operation};
use crate::ecs::ActiveWorkspaceMarker;
use crate::ecs::layout::LayoutStrip;
use crate::events::Event;

use super::*;

fn active_virtual_index(world: &mut World) -> u32 {
    let mut query = world.query_filtered::<&LayoutStrip, With<ActiveWorkspaceMarker>>();
    query
        .single(world)
        .expect("exactly one active strip")
        .virtual_index
}

#[test]
fn test_virtual_south_focuses_stack_sibling_before_switching_workspace() {
    let commands = vec![
        Event::MenuOpened { window_id: 0 },
        Event::Command {
            command: Command::Window(Operation::Focus(Direction::East)),
        },
        Event::Command {
            command: Command::Window(Operation::Stack(true)),
        },
        // Jump to VW1 and straight back so it exists for the eventual
        // fallback switch, without relying on auto-create being enabled.
        Event::Command {
            command: Command::Window(Operation::VirtualNumber(1)),
        },
        Event::Command {
            command: Command::Window(Operation::VirtualNumber(0)),
        },
        // Focus window 0 (top of the 2-item stack), then press South twice.
        Event::Command {
            command: Command::Window(Operation::Focus(Direction::First)),
        },
        Event::Command {
            command: Command::Window(Operation::Virtual(Direction::South)),
        },
        Event::Command {
            command: Command::Window(Operation::Virtual(Direction::South)),
        },
    ];

    TestHarness::new()
        .with_windows(2)
        .on_iteration(6, |world, _state| {
            // First South: a sibling exists below window 0 in the stack, so
            // focus must move to it instead of switching virtual workspace.
            assert_focused!(world, 1);
            assert_eq!(
                active_virtual_index(world),
                0,
                "focusing a stack sibling must not switch virtual workspace"
            );
        })
        .on_iteration(7, |world, _state| {
            // Second South: window 1 is the last stack item, nothing left to
            // focus, so this must fall through to switching workspaces.
            assert_eq!(
                active_virtual_index(world),
                1,
                "with nothing left to focus, South must fall through to switching workspace"
            );
        })
        .run(commands);
}

#[test]
fn test_virtual_north_switches_workspace_immediately_without_a_stack() {
    let commands = vec![
        Event::MenuOpened { window_id: 0 },
        Event::Command {
            command: Command::Window(Operation::VirtualNumber(1)),
        },
        Event::Command {
            command: Command::Window(Operation::Virtual(Direction::North)),
        },
    ];

    TestHarness::new()
        .with_windows(1)
        .on_iteration(2, |world, _state| {
            assert_eq!(
                active_virtual_index(world),
                0,
                "with no stack sibling to focus, North must switch virtual workspace as before"
            );
        })
        .run(commands);
}
