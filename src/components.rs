use crate::*;

/// Wrapper around any type to make it a `Component`.
#[derive(new)]
pub struct Comp<T>(pub T);
impl<T: Send + Sync + 'static> Component for Comp<T> {
    type Storage = DenseVecStorage<Self>;
}

/// A single colored letter sprite.
#[derive(Component)]
pub struct Sprite {
    pub glyph: u16,
    pub fg: RGBA,
    pub bg: RGBA,
}

/// The index of a 2d sprite. Created from `SpriteSheet`'s index.
#[derive(Component)]
pub struct SpriteIndex(pub usize);

/// A text-based sprite that is multiple tiles wide/high.
#[derive(Component, new)]
pub struct MultiSprite {
    pub tile: MultiTileSprite,
}

/// The path calculated by the Ai that it will follow.
#[derive(Component, new)]
pub struct AiPath {
    pub path: NavigationPath,
}

/// Indicates that the ai should calculate an AiPath from the current position
/// towards this destination.
#[derive(Component, new)]
pub struct AiDestination {
    pub target: Point,
}

/// Indicates that the ai should calculate an AiPath from the current position
/// towards this destination.
#[derive(Component, new)]
pub struct GotoStraight {
    pub target: Point,
    pub speed: f32,
}

/// Indicates that the ai should calculate an AiPath from the current position
/// towards this entity's position.
#[derive(Component, new)]
pub struct GotoEntity {
    pub entity: Entity,
    pub speed: f32,
}

/// Collision of a single tile entity
#[derive(Component)]
pub struct Collision;
/// Collision of a multi tile entity. Not necessarily colliding everywhere.
/// Can be both used as a global resource and as a component for individual entities.
#[derive(Component)]
pub struct CollisionMap {
    bitset: BitSet,
    width: u32,
    height: u32,
}

impl CollisionMap {
    /// Create a new collision map.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            bitset: BitSet::with_capacity(width * height),
            width,
            height,
        }
    }

    /// Enable collision at the given position.
    pub fn set(&mut self, x: u32, y: u32) {
        self.bitset.add(self.index_of(x, y));
    }

    /// Disable collision at the given position.
    pub fn unset(&mut self, x: u32, y: u32) {
        self.bitset.remove(self.index_of(x, y));
    }

    /// Checks if collision is enabled at the given position.
    pub fn is_set(&self, x: u32, y: u32) -> bool {
        self.bitset.contains(self.index_of(x, y))
    }

    /// Gives the size of the collision map.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Erase the collision map.
    pub fn clear(&mut self) {
        self.bitset.clear();
    }

    pub(crate) fn index_of(&self, x: u32, y: u32) -> u32 {
        let idx = y * self.width + x;
        assert!(idx <= self.width * self.height - 1);
        idx
    }

    pub(crate) fn position_of(&self, idx: u32) -> (u32, u32) {
        assert!(self.width > 0);
        assert!(self.height > 0);
        (idx % self.width, idx / self.width)
    }
}

impl BaseMap for CollisionMap {
    fn is_opaque(&self, idx: usize) -> bool {
        self.bitset.contains(idx as u32)
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut o = SmallVec::new();
        //println!("idx: {}", idx);
        // right
        if (idx % self.width as usize) < (self.width as usize - 1) {
            let n = idx + 1;
            if !self.is_opaque(n) {
                //println!("ADDING AT {},{}, while it is {} opaque.", self.position_of(idx as u32).0, self.position_of(idx as u32).1, self.is_opaque(idx));
                o.push((n, 1.0));
            }
        }
        // left
        if (idx % self.width as usize) > 0 {
            let n = idx - 1;
            if !self.is_opaque(n) {
                o.push((n, 1.0));
            }
        }
        // down
        if (idx / self.width as usize) < (self.height as usize - 1) {
            let n = idx + self.width as usize;
            if !self.is_opaque(n) {
                o.push((n, 1.0));
            }
        }
        // up
        if idx >= (self.width as usize) {
            let n = idx - self.width as usize;
            if !self.is_opaque(n) {
                o.push((n, 1.0));
            }
        }
        o
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let (x1, y1) = self.position_of(idx1 as u32);
        let (x2, y2) = self.position_of(idx2 as u32);
        ((x2 as f32 - x1 as f32).powf(2.0) + (y2 as f32 - y1 as f32).powf(2.0)).sqrt()
    }
}

/// Used to change the visible space of the world on screen.
#[derive(new)]
pub struct Camera {
    pub position: Point,
    pub size: Point,
}

/// A direction towards one of the 3d axis.
#[derive(Debug, Clone, Copy, Component)]
pub enum Direction {
    North,
    East,
    South,
    West,
    Up,
    Down,
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn collision_map_set_unset_clear() {
        let mut map = CollisionMap::new(5, 5);
        assert!(!map.is_set(3, 3));
        map.set(3, 3);
        assert!(map.is_set(3, 3));
        map.unset(3, 3);
        assert!(!map.is_set(3, 3));
        map.set(3, 3);
        map.clear();
        assert!(!map.is_set(3, 3));
    }
    #[test]
    fn small_map() {
        let _ = CollisionMap::new(0, 0);
        let mut map = CollisionMap::new(1, 1);
        map.set(0, 0);
        assert!(map.is_set(0, 0));
    }
    #[test]
    fn huge_map() {
        let mut map = CollisionMap::new(1000, 1000);
        map.set(999, 999);
    }
    #[test]
    #[should_panic]
    fn small_map_out_of_bounds() {
        let mut map = CollisionMap::new(0, 0);
        map.set(0, 0);
        assert!(map.is_set(0, 0));
    }
    #[test]
    #[should_panic]
    fn big_map_out_of_bounds() {
        let mut map = CollisionMap::new(1000, 1000);
        map.set(1000, 1000);
        assert!(map.is_set(1000, 1000));
        map.set(9999, 1000);
        assert!(map.is_set(9999, 1000));
    }
}