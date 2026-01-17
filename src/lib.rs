#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_must_use)]
#![warn(static_mut_refs)]

use std::sync::Mutex;
use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::{jint, jboolean, jintArray};
use once_cell::sync::Lazy;

use crate::rsmod::{
    Blocked, can_travel, CollisionStrategies, CollisionType, find_naive_path, has_line_of_sight,
    has_line_of_walk, Indoors, line_of_sight, line_of_walk, LineOfSight, LocAngle, LocLayer,
    LocShape, Normal, Outdoors, PathFinder,
};
use crate::rsmod::collision::collision::CollisionFlagMap;
use crate::rsmod::collision_flag::CollisionFlag;
use crate::rsmod::reach_strategy::ReachStrategy;

pub mod rsmod;

static COLLISION_FLAGS: Lazy<Mutex<CollisionFlagMap>> =
    Lazy::new(|| Mutex::new(CollisionFlagMap::new()));

static PATHFINDER: Lazy<Mutex<PathFinder>> =
    Lazy::new(|| Mutex::new(PathFinder::new()));

fn vec_to_jint_array(env: &JNIEnv, data: Vec<u32>) -> jintArray {
    let jint_vec: Vec<jint> = data.iter().map(|&v| v as jint).collect();
    let array = env.new_int_array(jint_vec.len() as i32).unwrap();
    env.set_int_array_region(&array, 0, &jint_vec).unwrap();
    array.as_raw()
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_findPath(
    env: JNIEnv,
    _class: JClass,
    y: jint,
    srcX: jint,
    srcZ: jint,
    destX: jint,
    destZ: jint,
    srcSize: jint,
    destWidth: jint,
    destHeight: jint,
    angle: jint,
    shape: jint,
    moveNear: jboolean,
    blockAccessFlags: jint,
    maxWaypoints: jint,
    collision: jint,
) -> jintArray  {
    vec_to_jint_array(&env, PATHFINDER.lock().unwrap().find_path(
        &*COLLISION_FLAGS.lock().unwrap(),
        y,
        srcX,
        srcZ,
        destX,
        destZ,
        srcSize as u8,
        destWidth as u8,
        destHeight as u8,
        angle as u8,
        shape as i8,
        moveNear != 0,
        blockAccessFlags as u8,
        maxWaypoints as u8,
        &get_collision_strategy(match collision {
            0 => CollisionType::NORMAL,
            1 => CollisionType::BLOCKED,
            2 => CollisionType::INDOORS,
            3 => CollisionType::OUTDOORS,
            4 => CollisionType::LINE_OF_SIGHT,
            _ => CollisionType::NORMAL,
        }),
    ))
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_findNaivePath(
    env: JNIEnv,
    _class: JClass,
    y: jint,
    srcX: jint,
    srcZ: jint,
    destX: jint,
    destZ: jint,
    srcWidth: jint,
    srcHeight: jint,
    destWidth: jint,
    destHeight: jint,
    extraFlag: jint,
    collision: jint,
) -> jintArray {
    let flags = COLLISION_FLAGS.lock().unwrap();
    vec_to_jint_array(&env,  find_naive_path(
        &*flags,
        y,
        srcX,
        srcZ,
        destX,
        destZ,
        srcWidth as u8,
        srcHeight as u8,
        destWidth as u8,
        destHeight as u8,
        extraFlag as u32,
        &get_collision_strategy(match collision {
            0 => CollisionType::NORMAL,
            1 => CollisionType::BLOCKED,
            2 => CollisionType::INDOORS,
            3 => CollisionType::OUTDOORS,
            4 => CollisionType::LINE_OF_SIGHT,
            _ => CollisionType::NORMAL,
        }),
    ))
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_changeFloor(
    _: JNIEnv,
    _class: JClass,
    x: jint,
    z: jint,
    y: jint,
    add: jboolean
) {
    if add != 0 {
        COLLISION_FLAGS.lock().unwrap().add(x, z, y, CollisionFlag::FLOOR as u32);
    } else {
        COLLISION_FLAGS.lock().unwrap().remove(x, z, y, CollisionFlag::FLOOR as u32);
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_changeLoc(
    _: JNIEnv,
    _class: JClass,
    x: jint,
    z: jint,
    y: jint,
    width: jint,
    length: jint,
    blockrange: jboolean,
    breakroutefinding: jboolean,
    add: jboolean,
) {
    let mut mask: u32 = CollisionFlag::LOC as u32;
    if blockrange != 0 {
        mask |= CollisionFlag::LOC_PROJ_BLOCKER as u32;
    }
    if breakroutefinding != 0 {
        mask |= CollisionFlag::LOC_ROUTE_BLOCKER as u32;
    }
    let area: i32 = width * length;
    if add != 0 {
        for index in 0..area {
            let dx: i32 = x + (index % width);
            let dz: i32 = z + (index / width);
            COLLISION_FLAGS.lock().unwrap().add(dx, dz, y, mask);
        }
    } else {
        for index in 0..area {
            let dx: i32 = x + (index % width);
            let dz: i32 = z + (index / width);
            COLLISION_FLAGS.lock().unwrap().remove(dx, dz, y, mask);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_changeNpc(
    _: JNIEnv,
    _class: JClass,
    x: jint,
    z: jint,
    y: jint,
    size: jint,
    add: jboolean
) {
    let mask: u32 = CollisionFlag::NPC as u32;
    let area: i32 = size * size;
    if add != 0 {
        for index in 0..area {
            let dx: i32 = x + (index % size);
            let dz: i32 = z + (index / size);
            COLLISION_FLAGS.lock().unwrap().add(dx, dz, y, mask);
        }
    } else {
        for index in 0..area {
            let dx: i32 = x + (index % size);
            let dz: i32 = z + (index / size);
            COLLISION_FLAGS.lock().unwrap().remove(dx, dz, y, mask);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_changePlayer(
    _: JNIEnv,
    _class: JClass,
    x: jint,
    z: jint,
    y: jint,
    size: jint,
    add: jboolean
) {
    let mask: u32 = CollisionFlag::PLAYER as u32;
    let area: i32 = size * size;
    if add != 0 {
        for index in 0..area {
            let dx: i32 = x + (index % size);
            let dz: i32 = z + (index / size);
            COLLISION_FLAGS.lock().unwrap().add(dx, dz, y, mask);
        }
    } else {
        for index in 0..area {
            let dx: i32 = x + (index % size);
            let dz: i32 = z + (index / size);
            COLLISION_FLAGS.lock().unwrap().remove(dx, dz, y, mask);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_changeRoof(
    _: JNIEnv,
    _class: JClass,
    x: jint,
    z: jint,
    y: jint,
    add: jboolean
) {
    if add != 0 {
        COLLISION_FLAGS.lock().unwrap().add(x, z, y, CollisionFlag::ROOF as u32);
    } else {
        COLLISION_FLAGS.lock().unwrap().remove(x, z, y, CollisionFlag::ROOF as u32);
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_changeWall(
    env: JNIEnv,
    _class: JClass,
    x: jint,
    z: jint,
    y: jint,
    angle: jint,
    shape: jint,
    blockrange: jboolean,
    breakroutefinding: jboolean,
    add: jboolean,
) {
    let shape = match shape {
        0 => LocShape::WALL_STRAIGHT,
        1 => LocShape::WALL_DIAGONAL_CORNER,
        2 => LocShape::WALL_L,
        3 => LocShape::WALL_SQUARE_CORNER,
        _ => LocShape::WALL_STRAIGHT,
    };

    match LocShape::from(shape) {
        LocShape::WALL_STRAIGHT => {
            Java_rsmod_PathFinder_changeWallStraight(env, _class, x, z, y, angle, blockrange, breakroutefinding, add)
        }
        LocShape::WALL_DIAGONAL_CORNER | LocShape::WALL_SQUARE_CORNER => {
            Java_rsmod_PathFinder_changeWallCorner(env, _class, x, z, y, angle, blockrange, breakroutefinding, add)
        }
        LocShape::WALL_L => Java_rsmod_PathFinder_changeWallL(env, _class, x, z, y, angle, blockrange, breakroutefinding, add),
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_changeWallStraight(
    env: JNIEnv,
    _class: JClass,
    x: jint,
    z: jint,
    y: jint,
    angle: jint,
    blockrange: jboolean,
    breakroutefinding: jboolean,
    add: jboolean,
) {
    let west: u32 = if breakroutefinding != 0 {
        CollisionFlag::WALL_WEST_ROUTE_BLOCKER
    } else if blockrange != 0 {
        CollisionFlag::WALL_WEST_PROJ_BLOCKER
    } else {
        CollisionFlag::WALL_WEST
    } as u32;
    let east: u32 = if breakroutefinding != 0 {
        CollisionFlag::WALL_EAST_ROUTE_BLOCKER
    } else if blockrange != 0 {
        CollisionFlag::WALL_EAST_PROJ_BLOCKER
    } else {
        CollisionFlag::WALL_EAST
    } as u32;
    let north: u32 = if breakroutefinding != 0 {
        CollisionFlag::WALL_NORTH_ROUTE_BLOCKER
    } else if blockrange != 0 {
        CollisionFlag::WALL_NORTH_PROJ_BLOCKER
    } else {
        CollisionFlag::WALL_NORTH
    } as u32;
    let south: u32 = if breakroutefinding != 0 {
        CollisionFlag::WALL_SOUTH_ROUTE_BLOCKER
    } else if blockrange != 0 {
        CollisionFlag::WALL_SOUTH_PROJ_BLOCKER
    } else {
        CollisionFlag::WALL_SOUTH
    } as u32;

    match LocAngle::from(angle as u8) {
        LocAngle::WEST => {
            if add != 0 {
                COLLISION_FLAGS.lock().unwrap().add(x, z, y, west);
                COLLISION_FLAGS.lock().unwrap().add(x - 1, z, y, east);
            } else {
                COLLISION_FLAGS.lock().unwrap().remove(x, z, y, west);
                COLLISION_FLAGS.lock().unwrap().remove(x - 1, z, y, east);
            }
        }
        LocAngle::NORTH => {
            if add != 0 {
                COLLISION_FLAGS.lock().unwrap().add(x, z, y, north);
                COLLISION_FLAGS.lock().unwrap().add(x, z + 1, y, south);
            } else {
                COLLISION_FLAGS.lock().unwrap().remove(x, z, y, north);
                COLLISION_FLAGS.lock().unwrap().remove(x, z + 1, y, south);
            }
        }
        LocAngle::EAST => {
            if add != 0 {
                COLLISION_FLAGS.lock().unwrap().add(x, z, y, east);
                COLLISION_FLAGS.lock().unwrap().add(x + 1, z, y, west);
            } else {
                COLLISION_FLAGS.lock().unwrap().remove(x, z, y, east);
                COLLISION_FLAGS.lock().unwrap().remove(x + 1, z, y, west);
            }
        }
        LocAngle::SOUTH => {
            if add != 0 {
                COLLISION_FLAGS.lock().unwrap().add(x, z, y, south);
                COLLISION_FLAGS.lock().unwrap().add(x, z - 1, y, north);
            } else {
                COLLISION_FLAGS.lock().unwrap().remove(x, z, y, south);
                COLLISION_FLAGS.lock().unwrap().remove(x, z - 1, y, north);
            }
        }
    }
    if breakroutefinding != 0 {
        return Java_rsmod_PathFinder_changeWallStraight(env, _class, x, z, y, angle, blockrange, 0, add);
    }
    if blockrange != 0 {
        return Java_rsmod_PathFinder_changeWallStraight(env, _class, x, z, y, angle, 0, 0, add);
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_changeWallCorner(
    env: JNIEnv,
    _class: JClass,
    x: jint,
    z: jint,
    y: jint,
    angle: jint,
    blockrange: jboolean,
    breakroutefinding: jboolean,
    add: jboolean,
) {
    let north_west: u32 = if breakroutefinding != 0 {
        CollisionFlag::WALL_NORTH_WEST_ROUTE_BLOCKER
    } else if blockrange != 0 {
        CollisionFlag::WALL_NORTH_WEST_PROJ_BLOCKER
    } else {
        CollisionFlag::WALL_NORTH_WEST
    } as u32;
    let south_east: u32 = if breakroutefinding != 0 {
        CollisionFlag::WALL_SOUTH_EAST_ROUTE_BLOCKER
    } else if blockrange != 0 {
        CollisionFlag::WALL_SOUTH_EAST_PROJ_BLOCKER
    } else {
        CollisionFlag::WALL_SOUTH_EAST
    } as u32;
    let north_east: u32 = if breakroutefinding != 0 {
        CollisionFlag::WALL_NORTH_EAST_ROUTE_BLOCKER
    } else if blockrange != 0 {
        CollisionFlag::WALL_NORTH_EAST_PROJ_BLOCKER
    } else {
        CollisionFlag::WALL_NORTH_EAST
    } as u32;
    let south_west: u32 = if breakroutefinding != 0 {
        CollisionFlag::WALL_SOUTH_WEST_ROUTE_BLOCKER
    } else if blockrange != 0 {
        CollisionFlag::WALL_SOUTH_WEST_PROJ_BLOCKER
    } else {
        CollisionFlag::WALL_SOUTH_WEST
    } as u32;

    match LocAngle::from(angle as u8) {
        LocAngle::WEST => {
            if add != 0 {
                COLLISION_FLAGS.lock().unwrap().add(x, z, y, north_west);
                COLLISION_FLAGS.lock().unwrap().add(x - 1, z + 1, y, south_east);
            } else {
                COLLISION_FLAGS.lock().unwrap().remove(x, z, y, north_west);
                COLLISION_FLAGS.lock().unwrap().remove(x - 1, z + 1, y, south_east);
            }
        }
        LocAngle::NORTH => {
            if add != 0 {
                COLLISION_FLAGS.lock().unwrap().add(x, z, y, north_east);
                COLLISION_FLAGS.lock().unwrap().add(x + 1, z + 1, y, south_west);
            } else {
                COLLISION_FLAGS.lock().unwrap().remove(x, z, y, north_east);
                COLLISION_FLAGS.lock().unwrap().remove(x + 1, z + 1, y, south_west);
            }
        }
        LocAngle::EAST => {
            if add != 0 {
                COLLISION_FLAGS.lock().unwrap().add(x, z, y, south_east);
                COLLISION_FLAGS.lock().unwrap().add(x + 1, z - 1, y, north_west);
            } else {
                COLLISION_FLAGS.lock().unwrap().remove(x, z, y, south_east);
                COLLISION_FLAGS.lock().unwrap().remove(x + 1, z - 1, y, north_west);
            }
        }
        LocAngle::SOUTH => {
            if add != 0 {
                COLLISION_FLAGS.lock().unwrap().add(x, z, y, south_west);
                COLLISION_FLAGS.lock().unwrap().add(x - 1, z - 1, y, north_east);
            } else {
                COLLISION_FLAGS.lock().unwrap().remove(x, z, y, south_west);
                COLLISION_FLAGS.lock().unwrap().remove(x - 1, z - 1, y, north_east);
            }
        }
    }
    if breakroutefinding != 0 {
        return Java_rsmod_PathFinder_changeWallCorner(env, _class, x, z, y, angle, blockrange, 0, add);
    }
    if blockrange != 0 {
        return Java_rsmod_PathFinder_changeWallCorner(env, _class, x, z, y, angle, 0, 0, add);
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_changeWallL(
    env: JNIEnv,
    _class: JClass,
    x: jint,
    z: jint,
    y: jint,
    angle: jint,
    blockrange: jboolean,
    breakroutefinding: jboolean,
    add: jboolean,
) {
    let west: u32 = if breakroutefinding != 0 {
        CollisionFlag::WALL_WEST_ROUTE_BLOCKER
    } else if blockrange != 0 {
        CollisionFlag::WALL_WEST_PROJ_BLOCKER
    } else {
        CollisionFlag::WALL_WEST
    } as u32;
    let east: u32 = if breakroutefinding != 0 {
        CollisionFlag::WALL_EAST_ROUTE_BLOCKER
    } else if blockrange != 0 {
        CollisionFlag::WALL_EAST_PROJ_BLOCKER
    } else {
        CollisionFlag::WALL_EAST
    } as u32;
    let north: u32 = if breakroutefinding != 0 {
        CollisionFlag::WALL_NORTH_ROUTE_BLOCKER
    } else if blockrange != 0 {
        CollisionFlag::WALL_NORTH_PROJ_BLOCKER
    } else {
        CollisionFlag::WALL_NORTH
    } as u32;
    let south: u32 = if breakroutefinding != 0 {
        CollisionFlag::WALL_SOUTH_ROUTE_BLOCKER
    } else if blockrange != 0 {
        CollisionFlag::WALL_SOUTH_PROJ_BLOCKER
    } else {
        CollisionFlag::WALL_SOUTH
    } as u32;

    match LocAngle::from(angle as u8) {
        LocAngle::WEST => {
            if add != 0 {
                COLLISION_FLAGS.lock().unwrap().add(x, z, y, north | west);
                COLLISION_FLAGS.lock().unwrap().add(x - 1, z, y, east);
                COLLISION_FLAGS.lock().unwrap().add(x, z + 1, y, south);
            } else {
                COLLISION_FLAGS.lock().unwrap().remove(x, z, y, north | west);
                COLLISION_FLAGS.lock().unwrap().remove(x - 1, z, y, east);
                COLLISION_FLAGS.lock().unwrap().remove(x, z + 1, y, south);
            }
        }
        LocAngle::NORTH => {
            if add != 0 {
                COLLISION_FLAGS.lock().unwrap().add(x, z, y, north | east);
                COLLISION_FLAGS.lock().unwrap().add(x, z + 1, y, south);
                COLLISION_FLAGS.lock().unwrap().add(x + 1, z, y, west);
            } else {
                COLLISION_FLAGS.lock().unwrap().remove(x, z, y, north | east);
                COLLISION_FLAGS.lock().unwrap().remove(x, z + 1, y, south);
                COLLISION_FLAGS.lock().unwrap().remove(x + 1, z, y, west);
            }
        }
        LocAngle::EAST => {
            if add != 0 {
                COLLISION_FLAGS.lock().unwrap().add(x, z, y, south | east);
                COLLISION_FLAGS.lock().unwrap().add(x + 1, z, y, west);
                COLLISION_FLAGS.lock().unwrap().add(x, z - 1, y, north);
            } else {
                COLLISION_FLAGS.lock().unwrap().remove(x, z, y, south | east);
                COLLISION_FLAGS.lock().unwrap().remove(x + 1, z, y, west);
                COLLISION_FLAGS.lock().unwrap().remove(x, z - 1, y, north);
            }
        }
        LocAngle::SOUTH => {
            if add != 0 {
                COLLISION_FLAGS.lock().unwrap().add(x, z, y, south | west);
                COLLISION_FLAGS.lock().unwrap().add(x, z - 1, y, north);
                COLLISION_FLAGS.lock().unwrap().add(x - 1, z, y, east);
            } else {
                COLLISION_FLAGS.lock().unwrap().remove(x, z, y, south | west);
                COLLISION_FLAGS.lock().unwrap().remove(x, z - 1, y, north);
                COLLISION_FLAGS.lock().unwrap().remove(x - 1, z, y, east);
            }
        }
    }
    if breakroutefinding != 0 {
        return Java_rsmod_PathFinder_changeWallL(env, _class, x, z, y, angle, blockrange, 0, add);
    }
    if blockrange != 0 {
        return Java_rsmod_PathFinder_changeWallL(env, _class, x, z, y, angle, 0, 0, add);
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_allocateIfAbsent(
    _: JNIEnv,
    _class: JClass,
    x: jint,
    z: jint,
    y: jint
) {
    COLLISION_FLAGS.lock().unwrap().allocate_if_absent(x, z, y);
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_deallocateIfPresent(
    _: JNIEnv,
    _class: JClass,
    x: jint,
    z: jint,
    y: jint
) {
    COLLISION_FLAGS.lock().unwrap().deallocate_if_present(x, z, y);
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_isZoneAllocated(
    _: JNIEnv,
    _class: JClass,
    x: jint,
    z: jint,
    y: jint
) -> jboolean {
    if COLLISION_FLAGS.lock().unwrap().is_zone_allocated(x, z, y) { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_isFlagged(
    _: JNIEnv,
    _class: JClass,
    x: jint,
    z: jint,
    y: jint,
    masks: jint
) -> jboolean {
    if COLLISION_FLAGS.lock().unwrap().is_flagged(x, z, y, masks as u32) { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_canTravel(
    _: JNIEnv,
    _class: JClass,
    y: jint,
    x: jint,
    z: jint,
    offsetX: jint,
    offsetZ: jint,
    size: jint,
    extraFlag: jint,
    collision: jint,
) -> jboolean {
    let flags = COLLISION_FLAGS.lock().unwrap();
    if can_travel(
        &*flags,
        y,
        x,
        z,
        offsetX as i8,
        offsetZ as i8,
        size as u8,
        extraFlag as u32,
        &get_collision_strategy(match collision {
            0 => CollisionType::NORMAL,
            1 => CollisionType::BLOCKED,
            2 => CollisionType::INDOORS,
            3 => CollisionType::OUTDOORS,
            4 => CollisionType::LINE_OF_SIGHT,
            _ => CollisionType::NORMAL,
        }),
    ) { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_hasLineOfSight(
    _: JNIEnv,
    _class: JClass,
    y: jint,
    srcX: jint,
    srcZ: jint,
    destX: jint,
    destZ: jint,
    srcWidth: jint,
    srcHeight: jint,
    destWidth: jint,
    destHeight: jint,
    extraFlag: jint,
) -> jboolean {
    let flags = COLLISION_FLAGS.lock().unwrap();
    if has_line_of_sight(
        &*flags,
        y,
        srcX,
        srcZ,
        destX,
        destZ,
        srcWidth as u8,
        srcHeight as u8,
        destWidth as u8,
        destHeight as u8,
        extraFlag as u32,
    ) { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_hasLineOfWalk(
    _: JNIEnv,
    _class: JClass,
    y: jint,
    srcX: jint,
    srcZ: jint,
    destX: jint,
    destZ: jint,
    srcWidth: jint,
    srcHeight: jint,
    destWidth: jint,
    destHeight: jint,
    extraFlag: jint,
) -> jboolean {
    let flags = COLLISION_FLAGS.lock().unwrap();
    if has_line_of_walk(
        &*flags,
        y,
        srcX,
        srcZ,
        destX,
        destZ,
        srcWidth as u8,
        srcHeight as u8,
        destWidth as u8,
        destHeight as u8,
        extraFlag as u32,
    ) { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_lineOfSight(
    env: JNIEnv,
    _class: JClass,
    y: jint,
    srcX: jint,
    srcZ: jint,
    destX: jint,
    destZ: jint,
    srcWidth: jint,
    srcHeight: jint,
    destWidth: jint,
    destHeight: jint,
    extraFlag: jint,
) -> jintArray {
    let flags = COLLISION_FLAGS.lock().unwrap();
    vec_to_jint_array(&env, line_of_sight(
        &*flags,
        y,
        srcX,
        srcZ,
        destX,
        destZ,
        srcWidth as u8,
        srcHeight as u8,
        destWidth as u8,
        destHeight as u8,
        extraFlag as u32,
    ))
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_lineOfWalk(
    env: JNIEnv,
    _class: JClass,
    y: jint,
    srcX: jint,
    srcZ: jint,
    destX: jint,
    destZ: jint,
    srcWidth: jint,
    srcHeight: jint,
    destWidth: jint,
    destHeight: jint,
    extraFlag: jint,
) -> jintArray {
    let flags = COLLISION_FLAGS.lock().unwrap();
    vec_to_jint_array(&env, line_of_walk(
        &*flags,
        y,
        srcX,
        srcZ,
        destX,
        destZ,
        srcWidth as u8,
        srcHeight as u8,
        destWidth as u8,
        destHeight as u8,
        extraFlag as u32,
    ))
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_reached(
    _: JNIEnv,
    _class: JClass,
    y: jint,
    srcX: jint,
    srcZ: jint,
    destX: jint,
    destZ: jint,
    destWidth: jint,
    destHeight: jint,
    srcSize: jint,
    angle: jint,
    shape: jint,
    blockAccessFlags: jint,
) -> jboolean {
    let flags = COLLISION_FLAGS.lock().unwrap();
    if ReachStrategy::reached(
        &*flags,
        y,
        srcX,
        srcZ,
        destX,
        destZ,
        destWidth as u8,
        destHeight as u8,
        srcSize as u8,
        angle as u8,
        shape as i8,
        blockAccessFlags as u8,
    ) { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder_locShapeLayer(
    _: JNIEnv,
    _class: JClass,
    shape: jint
) -> LocLayer {
    match LocShape::from(shape as i8) {
        LocShape::WALL_STRAIGHT
        | LocShape::WALL_DIAGONAL_CORNER
        | LocShape::WALL_L
        | LocShape::WALL_SQUARE_CORNER => LocLayer::WALL,

        LocShape::WALLDECOR_STRAIGHT_NOOFFSET
        | LocShape::WALLDECOR_STRAIGHT_OFFSET
        | LocShape::WALLDECOR_DIAGONAL_OFFSET
        | LocShape::WALLDECOR_DIAGONAL_NOOFFSET
        | LocShape::WALLDECOR_DIAGONAL_BOTH => LocLayer::WALL_DECOR,

        LocShape::WALL_DIAGONAL
        | LocShape::CENTREPIECE_STRAIGHT
        | LocShape::CENTREPIECE_DIAGONAL
        | LocShape::ROOF_STRAIGHT
        | LocShape::ROOF_DIAGONAL_WITH_ROOFEDGE
        | LocShape::ROOF_DIAGONAL
        | LocShape::ROOF_L_CONCAVE
        | LocShape::ROOF_L_CONVEX
        | LocShape::ROOF_FLAT
        | LocShape::ROOFEDGE_STRAIGHT
        | LocShape::ROOFEDGE_DIAGONAL_CORNER
        | LocShape::ROOFEDGE_L
        | LocShape::ROOFEDGE_SQUARE_CORNER => LocLayer::GROUND,

        LocShape::GROUND_DECOR => LocLayer::GROUND_DECOR,
    }
}

// this is only to test benchmarking lumbridge.
#[no_mangle]
pub unsafe extern "system" fn Java_rsmod_PathFinder___set(
    _: JNIEnv,
    _class: JClass,
    x: jint,
    z: jint,
    y: jint,
    mask: jint
) {
    COLLISION_FLAGS.lock().unwrap().set(x, z, y, mask as u32);
}

#[inline(always)]
fn get_collision_strategy(
    collision: CollisionType
) -> CollisionStrategies {
    match collision {
        CollisionType::NORMAL => CollisionStrategies::Normal(Normal),
        CollisionType::BLOCKED => CollisionStrategies::Blocked(Blocked),
        CollisionType::INDOORS => CollisionStrategies::Indoors(Indoors),
        CollisionType::OUTDOORS => CollisionStrategies::Outdoors(Outdoors),
        CollisionType::LINE_OF_SIGHT => CollisionStrategies::LineOfSight(LineOfSight),
    }
}
