use zerocopy_derive::{FromBytes, Immutable, IntoBytes};

#[derive(Default)] pub struct IsCastle;
#[derive(Default)] pub struct IsTower;
#[derive(Default)] pub struct IsHouse;
#[derive(Default)] pub struct IsKnight;

#[derive(Default)] pub struct HasCollision;


#[derive(Default, Copy, Clone, PartialEq, FromBytes, IntoBytes, Immutable)]
pub struct EntityId(pub u8);

impl EntityId {
    pub const CASTLE: Self = Self(1);
    pub const TOWER: Self = Self(2);
    pub const HOUSE: Self = Self(3);
    pub const KNIGHT: Self = Self(4);

    #[inline(always)]
    pub fn is_knight(self) -> bool { match self { Self::KNIGHT => true, _ => false } }
}
